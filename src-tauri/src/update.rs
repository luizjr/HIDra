// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

//! In-app updates.
//!
//! Most platforms are handled by `tauri-plugin-updater`, which downloads the
//! signed artifact named in the release's `latest.json`, verifies it against the
//! public key baked into the binary, and swaps it in. That covers Windows,
//! macOS and the Linux AppImage.
//!
//! It does not cover a Linux `.deb` or `.rpm` install, because replacing files
//! owned by the package manager behind its back would corrupt its database.
//! Those get the path in this module instead: download the new package and its
//! signature, verify the signature with the same key the plugin uses, and hand
//! the file to `dpkg`/`rpm` through `pkexec`, so the system asks for
//! authorisation exactly as it would for any other package install.

use crate::error::{DevError, Result};
use minisign_verify::{PublicKey, Signature};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

/// Where updates may be downloaded from. Anything else is refused before a byte
/// is written to disk, so a bad `latest.json` cannot point the installer at an
/// arbitrary file.
const RELEASE_PREFIX: &str = "https://github.com/luizjr/HIDra/releases/download/";

/// The release signing key, the same one `tauri-plugin-updater` verifies with.
/// It also lives in `tauri.conf.json` for the plugin; the test at the bottom of
/// this file keeps the two from drifting apart.
const UPDATER_PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDM2RjA1QzA3RTk2OTc1RjcKUldUM2RXbnBCMXp3TmhnOWtiaUU5dmFLVXVnN1k2VVZnRmlvdlNTSm1iTzZVSjNyZ2d2S2tvdzcK";

/// How this copy of HIDra was installed, which decides how it can update.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Install {
    /// Windows installer, macOS bundle, or a Linux AppImage: the updater plugin
    /// can replace it in place.
    Managed,
    /// Installed by dpkg. Updated by downloading the new .deb.
    Deb,
    /// Installed by rpm. Updated by downloading the new .rpm.
    Rpm,
    /// A `cargo build` in a source tree, a distro package we did not build, a
    /// Flatpak: we notify, but never touch it.
    Unknown,
}

#[derive(Serialize)]
pub struct UpdateSupport {
    pub install: Install,
    /// Whether HIDra can install an update itself on this system.
    pub can_install: bool,
    /// Release asset this system installs from, with `{version}` to fill in.
    /// The frontend turns it into a download URL; `install_linux_package`
    /// refuses anything that is not such a URL.
    pub asset_pattern: &'static str,
}

#[cfg(target_os = "linux")]
fn detect_install() -> Install {
    // An AppImage always exports this, and the plugin uses it to find itself.
    if std::env::var_os("APPIMAGE").is_some() {
        return Install::Managed;
    }
    let Ok(exe) = std::env::current_exe() else {
        return Install::Unknown;
    };
    let exe = exe.to_string_lossy().to_string();
    // Ask the package managers who owns this binary rather than guessing from
    // the path: /usr/bin/hidra could be either, or neither.
    if owns(&["dpkg", "-S"], &exe) {
        return Install::Deb;
    }
    if owns(&["rpm", "-qf"], &exe) {
        return Install::Rpm;
    }
    Install::Unknown
}

#[cfg(target_os = "linux")]
fn owns(cmd: &[&str], path: &str) -> bool {
    Command::new(cmd[0])
        .args(&cmd[1..])
        .arg(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn detect_install() -> Install {
    Install::Managed
}

/// What the frontend needs to decide which update path to offer.
#[tauri::command]
pub fn update_support() -> UpdateSupport {
    let install = detect_install();
    UpdateSupport {
        install,
        can_install: !matches!(install, Install::Unknown),
        asset_pattern: match install {
            Install::Deb => "HIDra_{version}_amd64.deb",
            Install::Rpm => "HIDra-{version}-1.x86_64.rpm",
            Install::Managed if cfg!(target_os = "windows") => "HIDra_{version}_x64-setup.exe",
            Install::Managed if cfg!(target_os = "macos") => "HIDra_{version}_universal.dmg",
            _ => "HIDra_{version}_amd64.AppImage",
        },
    }
}

/// Download a package and its signature, verify, install with pkexec.
///
/// `url` must be a release asset of this repository; the signature lives beside
/// it as `<url>.sig`, produced by the same key as every other update artifact.
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn install_linux_package(url: String) -> Result<()> {
    let install = detect_install();
    if !matches!(install, Install::Deb | Install::Rpm) {
        return Err(DevError::Update(
            "esta instalação não é gerenciada por dpkg/rpm".into(),
        ));
    }
    if !url.starts_with(RELEASE_PREFIX) {
        return Err(DevError::Update(format!(
            "recusando baixar de fora das releases do projeto: {url}"
        )));
    }

    let package = download(&url).await?;
    let signature = download(&format!("{url}.sig")).await?;
    verify(&package, &signature)?;

    let name = url.rsplit('/').next().unwrap_or("hidra-update");
    let path: PathBuf = std::env::temp_dir().join(name);
    std::fs::write(&path, &package).map_err(|e| {
        DevError::Update(format!("não foi possível gravar {}: {e}", path.display()))
    })?;

    let (program, args): (&str, Vec<&str>) = match install {
        Install::Rpm => ("rpm", vec!["-U", "--force"]),
        _ => ("dpkg", vec!["-i"]),
    };
    let output = Command::new("pkexec")
        .arg(program)
        .args(&args)
        .arg(&path)
        .output()
        .map_err(|e| DevError::Update(format!("pkexec não pôde ser executado: {e}")))?;

    let _ = std::fs::remove_file(&path);
    if output.status.success() {
        return Ok(());
    }
    // 126/127 is polkit's "not authorised" / "cancelled".
    let code = output.status.code().unwrap_or(-1);
    if code == 126 || code == 127 {
        return Err(DevError::Update("instalação cancelada".into()));
    }
    Err(DevError::Update(format!(
        "{program} falhou: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn install_linux_package(url: String) -> Result<()> {
    let _ = url;
    Err(DevError::Update("only used on Linux".into()))
}

#[cfg(target_os = "linux")]
async fn download(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| DevError::Update(format!("download falhou: {e}")))?;
    if !response.status().is_success() {
        return Err(DevError::Update(format!(
            "download falhou com HTTP {}",
            response.status()
        )));
    }
    Ok(response
        .bytes()
        .await
        .map_err(|e| DevError::Update(format!("download interrompido: {e}")))?
        .to_vec())
}

/// Verify a downloaded package against the release signing key.
///
/// The key is the same one `tauri-plugin-updater` checks, so a package that
/// passes here came from the same release pipeline as any other update.
///
/// Both the key and the signature are stored the way Tauri stores them: base64
/// of the whole minisign file, not of the key material alone.
#[cfg(target_os = "linux")]
fn verify(package: &[u8], signature_b64: &[u8]) -> Result<()> {
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD;

    let key_file = b64
        .decode(UPDATER_PUBKEY.trim())
        .ok()
        .and_then(|raw| String::from_utf8(raw).ok())
        .ok_or_else(|| DevError::Update("chave pública ilegível".into()))?;
    // A minisign public key file is a comment line followed by the key itself.
    let key_line = key_file
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .ok_or_else(|| DevError::Update("chave pública vazia".into()))?;
    let pubkey = PublicKey::from_base64(key_line.trim())
        .map_err(|e| DevError::Update(format!("chave pública inválida: {e}")))?;

    let signature_file = std::str::from_utf8(signature_b64)
        .ok()
        .and_then(|text| b64.decode(text.trim()).ok())
        .and_then(|raw| String::from_utf8(raw).ok())
        .ok_or_else(|| DevError::Update("assinatura ilegível".into()))?;
    let signature = Signature::decode(&signature_file)
        .map_err(|e| DevError::Update(format!("assinatura ilegível: {e}")))?;

    pubkey.verify(package, &signature, false).map_err(|_| {
        DevError::Update("assinatura do pacote não confere — download recusado".into())
    })
}

#[cfg(test)]
mod tests {
    use super::UPDATER_PUBKEY;

    /// The plugin reads its key from tauri.conf.json and the Linux package path
    /// reads the constant above. A release signed with one and checked against
    /// the other would fail on users' machines, so they must never diverge.
    #[test]
    fn pubkey_matches_tauri_conf() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let in_conf = conf["plugins"]["updater"]["pubkey"].as_str().unwrap();
        assert_eq!(in_conf, UPDATER_PUBKEY);
    }

    /// A file signed by the release key must verify, and one byte of tampering
    /// must not. The fixture was produced by `tauri signer sign`, which is what
    /// CI runs, so this also pins the encoding the two sides agree on.
    #[cfg(target_os = "linux")]
    #[test]
    fn verifies_what_the_release_pipeline_signs() {
        let package = include_bytes!("../tests/fixtures/sample.bin");
        let signature = include_bytes!("../tests/fixtures/sample.bin.sig");
        super::verify(package, signature).expect("valid signature rejected");

        let mut tampered = package.to_vec();
        tampered[0] ^= 0xff;
        assert!(
            super::verify(&tampered, signature).is_err(),
            "tampering accepted"
        );
    }
}

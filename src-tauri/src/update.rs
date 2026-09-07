// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the UI needs to know about updating itself.
//!
//! The updating is `tauri-plugin-updater`'s job: it downloads the artifact
//! named in the release's `latest.json`, verifies it against the public key in
//! `tauri.conf.json`, and installs it the way that package format demands — in
//! place for an AppImage or a Windows install, and through `dpkg`/`rpm` under
//! `pkexec` for a Linux package, since the package manager owns those files.
//!
//! The one thing it cannot tell the UI is what to offer when it cannot help: a
//! `cargo build` in a source tree, or a build someone else packaged, must be
//! told a release exists and then left alone. That is what this module answers.

use serde::Serialize;
use tauri::utils::config::BundleType;
use tauri::utils::platform::bundle_type;

#[derive(Serialize)]
pub struct UpdateSupport {
    /// Package format this copy was installed from, for the UI to explain what
    /// pressing the button will do. `null` when HIDra did not package it.
    pub bundle: Option<&'static str>,
    /// Whether HIDra can install an update itself.
    pub can_install: bool,
    /// Release asset for this system, with `{version}` still to fill in, used
    /// for the download link when it cannot.
    pub asset_pattern: &'static str,
}

/// How this copy was installed.
///
/// The bundler stamps the format into the binary it packages, so this is what
/// HIDra was actually installed from — not a guess from the path it runs from.
#[tauri::command]
pub fn update_support() -> UpdateSupport {
    let bundle = bundle_type();
    UpdateSupport {
        bundle: bundle.as_ref().map(|b| match b {
            BundleType::Deb => "deb",
            BundleType::Rpm => "rpm",
            BundleType::AppImage => "appimage",
            BundleType::Msi => "msi",
            BundleType::Nsis => "exe",
            BundleType::App | BundleType::Dmg => "app",
        }),
        can_install: bundle.is_some(),
        asset_pattern: match &bundle {
            Some(BundleType::Deb) => "HIDra_{version}_amd64.deb",
            Some(BundleType::Rpm) => "HIDra-{version}-1.x86_64.rpm",
            Some(BundleType::Msi) => "HIDra_{version}_x64_en-US.msi",
            Some(BundleType::Nsis) => "HIDra_{version}_x64-setup.exe",
            Some(BundleType::App) | Some(BundleType::Dmg) => "HIDra_{version}_universal.dmg",
            Some(BundleType::AppImage) => "HIDra_{version}_amd64.AppImage",
            // No packaging of ours: point at whatever this platform installs.
            _ if cfg!(target_os = "windows") => "HIDra_{version}_x64-setup.exe",
            _ if cfg!(target_os = "macos") => "HIDra_{version}_universal.dmg",
            _ => "HIDra_{version}_amd64.deb",
        },
    }
}

// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Brahma Pro / K586RGB-PRO (SONiX SN32F248B, 0C45:5004) protocol.
//!
//! Captured live from the official Windows tool under Wine + a /dev/uhid mirror,
//! with the GUI driven by xdotool. Output reports ID 0x04, 64 bytes; 16-bit sum
//! of bytes[3..63] stored little-endian at [1],[2]. Writes are wrapped BEGIN..END.

use crate::error::{DevError, Result};
use crate::devices::Protocol;
use crate::hid::{self, Kind, Link};
use hidapi::HidApi;
use serde::Serialize;

// Commands
const CMD_BEGIN: u8 = 0x01;
const CMD_END: u8 = 0x02;
const CMD_READ_SETTINGS: u8 = 0x05;
const CMD_WRITE_PARAM: u8 = 0x06;
const CMD_READ_KEYMAP_FACTORY: u8 = 0x07; // factory default (immutable)
const CMD_READ_KEYMAP_ACTIVE: u8 = 0x0f; // current map — reflects writes
const CMD_WRITE_KEYMAP: u8 = 0x08;
const CMD_WRITE_CUSTOM: u8 = 0x11;
const CMD_READ_PROFILE: u8 = 0x03; // read the profile-select block (active profile at [10])
const CMD_SELECT_PROFILE: u8 = 0x04; // switch the active profile

// LED param offsets within a profile's settings block.
const P_MODE: u16 = 0x00;
const P_BRIGHT: u16 = 0x01;
const P_SPEED: u16 = 0x02;
const P_DIR: u16 = 0x03;
const P_FULLRGB: u16 = 0x04;
const P_COLOR: u16 = 0x05;

pub const EFFECT_CUSTOM: u8 = 0x14;
pub const BRIGHT_MAX: u8 = 5;
pub const SPEED_MAX: u8 = 5;

/// Per-profile base offset for the settings block (P1/P2/P3).
fn profile_base(profile: u8) -> u16 {
    match profile {
        2 => 0x2A,
        3 => 0x54,
        _ => 0x00,
    }
}

fn checksum(buf: &mut [u8; 64]) {
    let s: u32 = buf[3..].iter().map(|&b| b as u32).sum();
    buf[1] = (s & 0xFF) as u8;
    buf[2] = ((s >> 8) & 0xFF) as u8;
}

fn frame(cmd: u8, off: u16, data: &[u8]) -> [u8; 64] {
    let mut b = [0u8; 64];
    b[0] = 0x04;
    b[3] = cmd;
    b[4] = data.len() as u8;
    b[5] = (off & 0xFF) as u8;
    b[6] = (off >> 8) as u8;
    b[8..8 + data.len()].copy_from_slice(data);
    checksum(&mut b);
    b
}

// BEGIN/END are just an empty frame with the command byte, exactly as captured:
// BEGIN = 04 01 00 01 00 00 00 00 …, END = 04 02 00 02 00 00 00 00 …
fn begin() -> [u8; 64] {
    frame(CMD_BEGIN, 0, &[])
}
fn end() -> [u8; 64] {
    frame(CMD_END, 0, &[])
}

fn read_block(link: &Link, cmd: u8, off: u16, len: u8) -> Result<Vec<u8>> {
    // Reply echoes cmd/off with data at [8..8+len].
    match link.kb_query(&frame(cmd, off, &vec![0u8; len as usize]), cmd)? {
        Some(reply) => Ok(reply[8..8 + len as usize].to_vec()),
        None => Err(DevError::NoResponse),
    }
}

// ---- LED parameters ----------------------------------------------------

fn write_param(link: &Link, profile: u8, param: u16, data: &[u8]) -> Result<()> {
    link.kb_write(&frame(CMD_WRITE_PARAM, profile_base(profile) + param, data))?;
    Ok(())
}

pub fn set_effect(
    api: &HidApi,
    profile: u8,
    effect: u8,
    r: u8,
    g: u8,
    b: u8,
    brightness: u8,
    speed: u8,
    direction: u8,
    full_rgb: bool,
) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_MODE, &[effect])?;
    write_param(&link, profile, P_BRIGHT, &[brightness.min(BRIGHT_MAX)])?;
    write_param(&link, profile, P_SPEED, &[speed.clamp(1, SPEED_MAX)])?;
    write_param(&link, profile, P_DIR, &[if direction != 0 { 0xFF } else { 0x00 }])?;
    write_param(&link, profile, P_FULLRGB, &[full_rgb as u8])?;
    write_param(&link, profile, P_COLOR, &[r, g, b])?;
    Ok(())
}

pub fn set_brightness(api: &HidApi, profile: u8, v: u8) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_BRIGHT, &[v.min(BRIGHT_MAX)])
}
pub fn set_speed(api: &HidApi, profile: u8, v: u8) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_SPEED, &[v.clamp(1, SPEED_MAX)])
}
pub fn set_direction(api: &HidApi, profile: u8, left: bool) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_DIR, &[if left { 0xFF } else { 0x00 }])
}
pub fn set_full_rgb(api: &HidApi, profile: u8, on: bool) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_FULLRGB, &[on as u8])
}
pub fn set_color(api: &HidApi, profile: u8, r: u8, g: u8, b: u8) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_COLOR, &[r, g, b])
}
pub fn set_mode(api: &HidApi, profile: u8, effect: u8) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_MODE, &[effect])
}

// ---- Active profile ----------------------------------------------------

/// The 44-byte profile-select block. Byte [10] is the active profile (0..2);
/// the rest is a fixed header captured from the official app. Confirmed on
/// hardware: writing this switches the profile the keyboard actually displays,
/// and it reads back via cmd 0x03.
#[rustfmt::skip]
const PROFILE_SELECT_BLOCK: [u8; 44] = [
    0x55, 0xaa, 0xff, 0x02, 0x45, 0x0c, 0x04, 0x50, 0x04, 0x01, 0x00, 0x38,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x11, 0x10, 0x12, 0x14, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Switch the profile the keyboard displays (1..3), like the official app.
pub fn set_active_profile(api: &HidApi, profile: u8) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    let mut block = PROFILE_SELECT_BLOCK;
    block[10] = profile.saturating_sub(1).min(2);
    link.kb_write(&frame(CMD_SELECT_PROFILE, 0, &block))?;
    Ok(())
}

/// Read which profile the keyboard is currently displaying (1..3).
fn read_active_profile(link: &Link) -> Option<u8> {
    read_block(link, CMD_READ_PROFILE, 0, 0x2c)
        .ok()
        .and_then(|b| b.get(10).map(|p| p + 1))
}

// ---- Effects catalogue -------------------------------------------------

#[derive(Serialize)]
pub struct Effect {
    pub id: u8,
    pub name: String,
    /// Whether this effect uses the single mode-colour and direction.
    pub has_color: bool,
    pub has_direction: bool,
}

pub fn effects() -> Vec<Effect> {
    // Order and PT names taken from the official app's combo box.
    // IDs = combo index + 1; "Customizável" = 0x14 (20).
    let names = [
        "Corredor", "A Nuvem", "Lâmina Veloz", "Spectrum", "Respiração", "Sólido",
        "Reativo", "Ondular", "Reativo (Horizontal)", "Florescer Gélido", "Rainbow",
        "Corrida das Sombras", "Tornado", "Recarregar", "A Matrix", "Surmount",
        "Passagem Dupla", "Vulto do Spectro",
    ];
    let mut v: Vec<Effect> = names
        .iter()
        .enumerate()
        .map(|(i, n)| Effect {
            id: (i + 1) as u8,
            name: (*n).into(),
            has_color: true,
            has_direction: matches!(i, 0 | 6 | 7 | 10),
        })
        .collect();
    v.push(Effect { id: EFFECT_CUSTOM, name: "Customizável".into(), has_color: false, has_direction: false });
    v
}

// ---- State read --------------------------------------------------------

#[derive(Serialize)]
pub struct KbState {
    pub present: bool,
    pub active: Option<u8>,
    pub effect: Option<u8>,
    pub brightness: Option<u8>,
    pub speed: Option<u8>,
    pub direction: Option<u8>,
    pub full_rgb: Option<bool>,
    pub color: Option<[u8; 3]>,
}

pub fn read_state(api: &HidApi, profile: u8) -> KbState {
    let Ok(link) = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64) else {
        return KbState { present: false, active: None, effect: None, brightness: None, speed: None, direction: None, full_rgb: None, color: None };
    };
    let active = read_active_profile(&link);
    match read_block(&link, CMD_READ_SETTINGS, profile_base(profile), 0x38) {
        Ok(d) if d.len() >= 8 => KbState {
            present: true,
            active,
            effect: Some(d[0]),
            brightness: Some(d[1]),
            speed: Some(d[2]),
            direction: Some(if d[3] != 0 { 1 } else { 0 }),
            full_rgb: Some(d[4] != 0),
            color: Some([d[5], d[6], d[7]]),
        },
        _ => KbState { present: true, active, effect: None, brightness: None, speed: None, direction: None, full_rgb: None, color: None },
    }
}

// ---- Keymap (remap) ----------------------------------------------------

/// Default keymap of THIS board (420 bytes = 140 × 3), captured from hardware.
/// Used to restore defaults and as the base when reading fails.
#[rustfmt::skip]
pub const DEFAULT_KEYMAP: [u8; 420] = [
    0x02, 0x02, 0x29, 0x02, 0x02, 0x35, 0x02, 0x02, 0x2b, 0x02, 0x02, 0x39,
    0x02, 0x01, 0x02, 0x02, 0x01, 0x01, 0x02, 0x02, 0x3f, 0x02, 0x02, 0x00,
    0x02, 0x02, 0x1e, 0x02, 0x02, 0x14, 0x02, 0x02, 0x04, 0x02, 0x02, 0x64,
    0x02, 0x01, 0x08, 0x02, 0x02, 0x40, 0x02, 0x02, 0x3a, 0x02, 0x02, 0x1f,
    0x02, 0x02, 0x1a, 0x02, 0x02, 0x16, 0x02, 0x02, 0x1d, 0x02, 0x01, 0x04,
    0x02, 0x02, 0x41, 0x02, 0x02, 0x3b, 0x02, 0x02, 0x20, 0x02, 0x02, 0x08,
    0x02, 0x02, 0x07, 0x02, 0x02, 0x1b, 0x02, 0x02, 0x00, 0x02, 0x02, 0x42,
    0x02, 0x02, 0x3c, 0x02, 0x02, 0x21, 0x02, 0x02, 0x15, 0x02, 0x02, 0x09,
    0x02, 0x02, 0x06, 0x02, 0x01, 0x00, 0x02, 0x02, 0x43, 0x02, 0x02, 0x3d,
    0x02, 0x02, 0x22, 0x02, 0x02, 0x17, 0x02, 0x02, 0x0a, 0x02, 0x02, 0x19,
    0x02, 0x02, 0x00, 0x05, 0x19, 0x00, 0x02, 0x02, 0x3e, 0x02, 0x02, 0x23,
    0x02, 0x02, 0x1c, 0x02, 0x02, 0x0b, 0x02, 0x02, 0x05, 0x02, 0x02, 0x2c,
    0x05, 0x0f, 0x00, 0x02, 0x02, 0x3f, 0x02, 0x02, 0x24, 0x02, 0x02, 0x18,
    0x02, 0x02, 0x0d, 0x02, 0x02, 0x11, 0x02, 0x02, 0x56, 0x05, 0x0f, 0x01,
    0x02, 0x02, 0x40, 0x02, 0x02, 0x25, 0x02, 0x02, 0x0c, 0x02, 0x02, 0x0e,
    0x02, 0x02, 0x10, 0x02, 0x02, 0x57, 0x05, 0x0f, 0x02, 0x02, 0x02, 0x41,
    0x02, 0x02, 0x26, 0x02, 0x02, 0x12, 0x02, 0x02, 0x0f, 0x02, 0x02, 0x36,
    0x02, 0x02, 0x58, 0x05, 0x0f, 0x03, 0x02, 0x02, 0x42, 0x02, 0x02, 0x27,
    0x02, 0x02, 0x13, 0x02, 0x02, 0x33, 0x02, 0x02, 0x37, 0x02, 0x01, 0x40,
    0x05, 0x25, 0x00, 0x02, 0x02, 0x43, 0x02, 0x02, 0x2d, 0x02, 0x02, 0x2f,
    0x02, 0x02, 0x34, 0x02, 0x02, 0x38, 0x05, 0x02, 0x02, 0x02, 0x00, 0x00,
    0x02, 0x02, 0x44, 0x02, 0x02, 0x2e, 0x02, 0x02, 0x30, 0x02, 0x02, 0x31,
    0x02, 0x02, 0x87, 0x02, 0x02, 0x65, 0x02, 0x00, 0x00, 0x02, 0x02, 0x45,
    0x02, 0x02, 0x2a, 0x02, 0x02, 0x31, 0x02, 0x02, 0x28, 0x02, 0x01, 0x20,
    0x02, 0x01, 0x10, 0x02, 0x00, 0x00, 0x02, 0x02, 0x46, 0x02, 0x02, 0x49,
    0x02, 0x02, 0x4c, 0x02, 0x02, 0x3a, 0x02, 0x02, 0x3d, 0x02, 0x02, 0x50,
    0x02, 0x00, 0x00, 0x02, 0x02, 0x47, 0x02, 0x02, 0x4a, 0x02, 0x02, 0x4d,
    0x02, 0x02, 0x3b, 0x02, 0x02, 0x52, 0x02, 0x02, 0x51, 0x02, 0x00, 0x00,
    0x02, 0x02, 0x48, 0x02, 0x02, 0x4b, 0x02, 0x02, 0x4e, 0x02, 0x02, 0x3c,
    0x02, 0x02, 0x3e, 0x02, 0x02, 0x4f, 0x02, 0x00, 0x00, 0x05, 0x18, 0x18,
    0x02, 0x02, 0x53, 0x02, 0x02, 0x5f, 0x02, 0x02, 0x5c, 0x02, 0x02, 0x59,
    0x03, 0xcd, 0x00, 0x03, 0xb5, 0x00, 0x02, 0x00, 0x00, 0x02, 0x02, 0x54,
    0x02, 0x02, 0x60, 0x02, 0x02, 0x5d, 0x02, 0x02, 0x5a, 0x02, 0x02, 0x62,
    0x02, 0x00, 0x00, 0x03, 0xb6, 0x00, 0x02, 0x02, 0x55, 0x02, 0x02, 0x61,
    0x02, 0x02, 0x5e, 0x02, 0x02, 0x5b, 0x02, 0x02, 0x63, 0x02, 0x00, 0x00,
];

const KEYMAP_OFFSETS: [(u16, u8); 8] = [
    (0x0000, 0x38), (0x0038, 0x38), (0x0070, 0x38), (0x00a8, 0x38),
    (0x00e0, 0x38), (0x0118, 0x38), (0x0150, 0x38), (0x0188, 0x1c),
];

/// Read the CURRENT keymap (cmd 0x0f). Confirmed on hardware: writes via
/// cmd 0x08 show up here, whereas cmd 0x07 always returns the factory map.
fn read_keymap(link: &Link) -> Result<[u8; 420]> {
    let mut out = [0u8; 420];
    let mut pos = 0usize;
    for (off, len) in KEYMAP_OFFSETS {
        let d = read_block(link, CMD_READ_KEYMAP_ACTIVE, off, len)?;
        out[pos..pos + len as usize].copy_from_slice(&d[..len as usize]);
        pos += len as usize;
    }
    Ok(out)
}

/// Read the factory keymap (cmd 0x07) — used only as a fallback/reference.
#[allow(dead_code)]
fn read_factory_keymap(link: &Link) -> Result<[u8; 420]> {
    let mut out = [0u8; 420];
    let mut pos = 0usize;
    for (off, len) in KEYMAP_OFFSETS {
        let d = read_block(link, CMD_READ_KEYMAP_FACTORY, off, len)?;
        out[pos..pos + len as usize].copy_from_slice(&d[..len as usize]);
        pos += len as usize;
    }
    Ok(out)
}

fn write_keymap(link: &Link, table: &[u8; 420]) -> Result<()> {
    link.kb_write(&begin())?;
    let mut pos = 0usize;
    for (off, len) in KEYMAP_OFFSETS {
        link.kb_write(&frame(CMD_WRITE_KEYMAP, off, &table[pos..pos + len as usize]))?;
        pos += len as usize;
    }
    link.kb_write(&end())?;
    Ok(())
}

/// Human-readable label for a HID keyboard usage code.
fn usage_label(u: u8) -> Option<&'static str> {
    Some(match u {
        0x04..=0x1d => return Some(LETTERS[(u - 0x04) as usize]),
        0x1e => "1", 0x1f => "2", 0x20 => "3", 0x21 => "4", 0x22 => "5",
        0x23 => "6", 0x24 => "7", 0x25 => "8", 0x26 => "9", 0x27 => "0",
        0x28 => "Enter", 0x29 => "Esc", 0x2a => "Backspace", 0x2b => "Tab", 0x2c => "Espaço",
        0x2d => "-", 0x2e => "=", 0x2f => "[", 0x30 => "]", 0x31 => "\\",
        0x33 => ";", 0x34 => "'", 0x35 => "`", 0x36 => ",", 0x37 => ".", 0x38 => "/",
        0x39 => "CapsLock",
        0x3a => "F1", 0x3b => "F2", 0x3c => "F3", 0x3d => "F4", 0x3e => "F5", 0x3f => "F6",
        0x40 => "F7", 0x41 => "F8", 0x42 => "F9", 0x43 => "F10", 0x44 => "F11", 0x45 => "F12",
        0x46 => "PrtSc", 0x47 => "ScrLk", 0x48 => "Pause", 0x49 => "Insert", 0x4a => "Home",
        0x4b => "PgUp", 0x4c => "Delete", 0x4d => "End", 0x4e => "PgDn",
        0x4f => "→", 0x50 => "←", 0x51 => "↓", 0x52 => "↑",
        0x53 => "NumLk", 0x54 => "Num/", 0x55 => "Num*", 0x56 => "Num-", 0x57 => "Num+",
        0x58 => "NumEnter", 0x59 => "Num1", 0x5a => "Num2", 0x5b => "Num3", 0x5c => "Num4",
        0x5d => "Num5", 0x5e => "Num6", 0x5f => "Num7", 0x60 => "Num8", 0x61 => "Num9",
        0x62 => "Num0", 0x63 => "Num.", 0x64 => "\\|", 0x65 => "Menu", 0x87 => "/? (ABNT)",
        _ => return None,
    })
}
const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
];

#[derive(Serialize)]
pub struct RemapKey {
    pub index: usize,
    pub label: String,
    pub raw: [u8; 3],
}

/// The list of physical keys that can be remapped, derived from the default
/// table so it always matches this board.
pub fn remap_keys() -> Vec<RemapKey> {
    let mut out = Vec::new();
    for i in 0..(DEFAULT_KEYMAP.len() / 3) {
        let e = [DEFAULT_KEYMAP[i * 3], DEFAULT_KEYMAP[i * 3 + 1], DEFAULT_KEYMAP[i * 3 + 2]];
        let label = match e {
            [0x02, 0x02, u] => usage_label(u).map(|s| s.to_string()),
            [0x02, 0x01, m] => Some(modifier_label(m).to_string()),
            [0x03, lo, _] => Some(media_label(lo).to_string()),
            _ => None,
        };
        if let Some(label) = label {
            out.push(RemapKey { index: i, label, raw: e });
        }
    }
    out
}

fn modifier_label(m: u8) -> &'static str {
    match m {
        0x01 => "Ctrl Esq", 0x02 => "Shift Esq", 0x04 => "Alt Esq", 0x08 => "Win Esq",
        0x10 => "Ctrl Dir", 0x20 => "Shift Dir", 0x40 => "Alt Dir", 0x80 => "Win Dir",
        _ => "Modificador",
    }
}
fn media_label(lo: u8) -> &'static str {
    match lo {
        0xcd => "Play/Pause", 0xb5 => "Próxima", 0xb6 => "Anterior", 0xb7 => "Parar",
        0xe2 => "Mudo", 0xe9 => "Vol +", 0xea => "Vol -",
        _ => "Multimídia",
    }
}

/// Encode a remap target chosen in the UI into a 3-byte option.
pub fn encode_remap(kind: &str, code: u8, code2: u8) -> [u8; 3] {
    match kind {
        "key" => [0x02, 0x02, code],
        "modifier" => [0x02, 0x01, code],
        "media" => [0x03, code, code2],
        "disable" => [0x02, 0x00, 0x00],
        _ => [0x02, 0x02, code],
    }
}

pub fn remap(api: &HidApi, index: usize, option: [u8; 3]) -> Result<()> {
    if index >= DEFAULT_KEYMAP.len() / 3 {
        return Err(DevError::Invalid("índice de tecla fora do intervalo".into()));
    }
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    let mut table = read_keymap(&link).unwrap_or(DEFAULT_KEYMAP);
    table[index * 3..index * 3 + 3].copy_from_slice(&option);
    write_keymap(&link, &table)
}

pub fn restore_keymap(api: &HidApi) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_keymap(&link, &DEFAULT_KEYMAP)
}

// ---- Custom per-key colours (effect "Customizável") --------------------
//
// Each key has a fixed LED byte-offset in the colour buffer (measured on this
// board — see src/keyboardLayout.ts). The official app paints one key at a time
// with `11 03 <off> <R G B>`, so we do the same: switch to the Customizável
// mode, then BEGIN, one 3-byte write per painted key, END.

#[derive(serde::Deserialize)]
pub struct CustomCell {
    pub off: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn set_custom_colors(api: &HidApi, profile: u8, cells: Vec<CustomCell>) -> Result<()> {
    let link = hid::open_speaking(api, Kind::Keyboard, Protocol::Sonix64)?;
    write_param(&link, profile, P_MODE, &[EFFECT_CUSTOM])?;
    link.kb_write(&begin())?;
    for c in &cells {
        link.kb_write(&frame(CMD_WRITE_CUSTOM, c.off, &[c.r, c.g, c.b]))?;
    }
    link.kb_write(&end())?;
    Ok(())
}

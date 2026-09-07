// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cobra Pro / M711-PRO (Compx 25A7:FA07/FA08) configuration protocol.
//!
//! Ported from the reverse-engineered `open-m711pro` (GPL-3.0), which validated
//! every byte against real hardware, cross-checked with `m913-ctl`/`mouse_m908`.
//! Feature Report ID 0x08, 17-byte packets; the sum of every packet and every
//! memory record settles at 0x55.

use crate::devices::Protocol;
use crate::error::{DevError, Result};
use crate::hid::{self, Kind, Link};
use hidapi::HidApi;
use serde::{Deserialize, Serialize};

const CKSUM_TARGET: u16 = 0x55;
const MAX_DATA: usize = 10;

const LED_ADDR: u16 = 0x00A0;
const POLLING_ADDR: u16 = 0x0000;
const DPI_ADDR: u16 = 0x000C;
const DPI_COLOR_ADDR: u16 = 0x002C;
const DPI_EFFECT_ADDR: u16 = 0x004C;
const BUTTON_ADDR: u16 = 0x0060;
const EXT_SLOT_BASE: u16 = 0x0100;
const EXT_SLOT_STRIDE: u16 = 0x20;

pub const DPI_STAGES: usize = 5;
pub const BUTTON_COUNT: usize = 16;
const DPI_MIN: u32 = 100;
const DPI_MAX: u32 = 16000;
const DPI_BYTE_MAX: u8 = 0xBD;

const RETRIES: usize = 4;

fn cksum(data: &[u8]) -> u8 {
    let sum: u32 = data.iter().map(|&b| b as u32).sum();
    ((CKSUM_TARGET.wrapping_sub((sum & 0xFF) as u16)) & 0xFF) as u8
}

fn record(values: &[u8]) -> Vec<u8> {
    let mut v = values.to_vec();
    v.push(cksum(&v));
    v
}

fn build_write(addr: u16, data: &[u8]) -> [u8; 17] {
    assert!(data.len() <= MAX_DATA);
    let mut p = [0u8; 17];
    p[0] = 0x08;
    p[1] = 0x07;
    p[2] = 0x00;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = data.len() as u8;
    p[6..6 + data.len()].copy_from_slice(data);
    let s: u16 = p[..16].iter().map(|&b| b as u16).sum();
    p[16] = ((CKSUM_TARGET.wrapping_sub(s & 0xFF)) & 0xFF) as u8;
    p
}

fn build_read(addr: u16, len: u8) -> [u8; 17] {
    let mut p = [0u8; 17];
    p[0] = 0x08;
    p[1] = 0x08;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = len;
    let s: u16 = p[..16].iter().map(|&b| b as u16).sum();
    p[16] = ((CKSUM_TARGET.wrapping_sub(s & 0xFF)) & 0xFF) as u8;
    p
}

fn build_commit() -> [u8; 17] {
    let mut p = [0u8; 17];
    p[0] = 0x08;
    p[1] = 0x04;
    let s: u16 = p[..16].iter().map(|&b| b as u16).sum();
    p[16] = ((CKSUM_TARGET.wrapping_sub(s & 0xFF)) & 0xFF) as u8;
    p
}

// ---- DPI encoding -------------------------------------------------------

pub fn dpi_to_byte(dpi: u32) -> u8 {
    let dpi = dpi.clamp(DPI_MIN, DPI_MAX);
    let n = (dpi * 3 + 125) / 250 - 1;
    n.min(DPI_BYTE_MAX as u32) as u8
}
pub fn byte_to_dpi(n: u8) -> u32 {
    ((n as u32 + 1) * 250 + 1) / 3
}

// ---- LED ---------------------------------------------------------------

pub const EFFECT_CYCLE: u8 = 0x00;
pub const EFFECT_BREATHING: u8 = 0x01;
pub const EFFECT_STEADY: u8 = 0x02;
pub const EFFECT_OFF: u8 = 0x04;

fn effect_id(name: &str) -> u8 {
    match name {
        "breathing" => EFFECT_BREATHING,
        "cycle" => EFFECT_CYCLE,
        "off" => EFFECT_OFF,
        _ => EFFECT_STEADY,
    }
}
fn effect_name(id: u8) -> &'static str {
    match id {
        EFFECT_CYCLE => "cycle",
        EFFECT_BREATHING => "breathing",
        EFFECT_OFF => "off",
        _ => "steady",
    }
}

fn build_led(effect: u8, r: u8, g: u8, b: u8, speed: u8, brightness: u8) -> [u8; 17] {
    build_write(LED_ADDR, &record(&[effect, r, g, b, speed, brightness]))
}

// ---- Buttons -----------------------------------------------------------

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Disabled,
    Mouse { button: String },
    Dpi { op: String },
    Fire { speed: u8, clicks: u8 },
    ThreeClick,
    RgbToggle,
    PollingSwitch,
    Media { usage: u16 },
    Key { usage: u8 },
    Combo { modifiers: u8, usage: u8 },
}

fn action_to_record(a: &Action) -> [u8; 3] {
    match a {
        Action::Disabled => [0x00, 0x00, 0x00],
        Action::Mouse { button } => {
            let mask = match button.as_str() {
                "left" => 0x01,
                "right" => 0x02,
                "middle" => 0x04,
                "back" => 0x08,
                "forward" => 0x10,
                _ => 0x01,
            };
            [0x01, mask, 0x00]
        }
        Action::Dpi { op } => {
            let p = match op.as_str() {
                "up" => 0x02,
                "down" => 0x03,
                _ => 0x01, // cycle
            };
            [0x02, p, 0x00]
        }
        Action::Fire { speed, clicks } => [0x04, *speed, *clicks],
        Action::ThreeClick => [0x04, 0x32, 0x03],
        Action::RgbToggle => [0x08, 0x00, 0x00],
        Action::PollingSwitch => [0x07, 0x00, 0x00],
        // Extended-slot actions carry the marker in the button record.
        Action::Media { .. } | Action::Key { .. } | Action::Combo { .. } => [0x05, 0x00, 0x00],
    }
}

fn build_buttons(actions: &[[u8; 3]; BUTTON_COUNT]) -> Vec<[u8; 17]> {
    let mut out = Vec::new();
    for i in (0..BUTTON_COUNT).step_by(2) {
        let mut data = Vec::new();
        data.extend(record(&actions[i]));
        data.extend(record(&actions[i + 1]));
        out.push(build_write(BUTTON_ADDR + 4 * i as u16, &data));
    }
    out
}

fn ext_slot_addr(index: usize) -> u16 {
    EXT_SLOT_BASE + EXT_SLOT_STRIDE * index as u16
}

fn build_events(index: usize, events: &[(u8, u16)]) -> Vec<[u8; 17]> {
    let mut data = vec![events.len() as u8];
    for &(t, code) in events {
        data.push(t);
        data.push((code & 0xFF) as u8);
        data.push((code >> 8) as u8);
    }
    data = record(&data);
    let base = ext_slot_addr(index);
    let mut out = Vec::new();
    let mut off = 0usize;
    while off < data.len() {
        let end = (off + MAX_DATA).min(data.len());
        out.push(build_write(base + off as u16, &data[off..end]));
        off += MAX_DATA;
    }
    out
}

fn ext_packets_for(index: usize, a: &Action) -> Vec<[u8; 17]> {
    match a {
        Action::Media { usage } => build_events(index, &[(0x82, *usage), (0x42, *usage)]),
        Action::Key { usage } => {
            build_events(index, &[(0x81, *usage as u16), (0x41, *usage as u16)])
        }
        Action::Combo { modifiers, usage } => build_events(
            index,
            &[
                (0x80, *modifiers as u16),
                (0x81, *usage as u16),
                (0x40, *modifiers as u16),
                (0x41, *usage as u16),
            ],
        ),
        _ => Vec::new(),
    }
}

// ---- device operations -------------------------------------------------

fn apply(link: &Link, packets: &[[u8; 17]]) -> Result<()> {
    for p in packets {
        for _ in 0..RETRIES {
            link.mouse_feature(p)?;
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    link.mouse_feature(&build_commit())?;
    Ok(())
}

/// Send a burst to EVERY mouse config interface present (2.4G dongle + USB-C),
/// as the official flow does: whichever path the mouse is actually listening on
/// takes it. Succeeds if at least one interface accepted every packet.
fn apply_all(api: &HidApi, packets: &[[u8; 17]]) -> Result<()> {
    let founds = crate::hid::find_all(api, Kind::Mouse);
    if founds.is_empty() {
        return Err(DevError::NotFound);
    }
    let mut ok = false;
    for f in founds
        .iter()
        .filter(|f| f.device.protocol == Protocol::Compx17)
    {
        if let Ok(link) = Link::open(api, f) {
            if apply(&link, packets).is_ok() {
                ok = true;
            }
        }
    }
    if ok {
        Ok(())
    } else {
        Err(DevError::NoResponse)
    }
}

/// Open whichever interface answers reads (prefers USB-C, always awake).
fn read_link(api: &HidApi) -> Result<Link> {
    hid::open_speaking(api, Kind::Mouse, Protocol::Compx17)
}

fn read_register(link: &Link, addr: u16, len: u8) -> Result<Option<Vec<u8>>> {
    for _ in 0..RETRIES {
        link.mouse_feature(&build_read(addr, len))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(700);
    while std::time::Instant::now() < deadline {
        if let Some(d) = link.mouse_read(150)? {
            if d.len() >= 6 + len as usize
                && d[0] == 0x09
                && d[1] == 0x08
                && d[3] == (addr >> 8) as u8
                && d[4] == (addr & 0xFF) as u8
            {
                return Ok(Some(d[6..6 + len as usize].to_vec()));
            }
        }
    }
    Ok(None)
}

// ---- public API (called from Tauri commands) ---------------------------

#[derive(Serialize)]
pub struct Led {
    pub effect: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub speed: u8,
    pub brightness: u8,
}

#[derive(Serialize)]
pub struct Stage {
    pub x: u32,
    pub y: u32,
    pub color: [u8; 3],
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct MouseState {
    pub present: bool,
    pub mode: Option<String>,
    pub led: Option<Led>,
    pub dpi: Option<Vec<Stage>>,
    pub dpi_effect: Option<bool>,
    pub polling: Option<u32>,
    pub buttons: Option<Vec<[u8; 3]>>,
}

pub fn read_state(api: &HidApi) -> MouseState {
    let found = crate::hid::find(api, Kind::Mouse);
    let Some(found) = found else {
        return MouseState {
            present: false,
            mode: None,
            led: None,
            dpi: None,
            dpi_effect: None,
            polling: None,
            buttons: None,
        };
    };
    let mode = Some(found.connection().to_string());
    let Ok(link) = Link::open(api, &found) else {
        return MouseState {
            present: true,
            mode,
            led: None,
            dpi: None,
            dpi_effect: None,
            polling: None,
            buttons: None,
        };
    };

    let led = read_register(&link, LED_ADDR, 8)
        .ok()
        .flatten()
        .map(|d| Led {
            effect: effect_name(d[0]).into(),
            r: d[1],
            g: d[2],
            b: d[3],
            speed: d[4],
            brightness: d[5],
        });

    // DPI table + colours (3 reads each, shapes 8/8/4).
    let dpi = read_stage_table(&link, DPI_ADDR).and_then(|axes| {
        let colors = read_stage_table(&link, DPI_COLOR_ADDR)?;
        Some(
            axes.iter()
                .zip(colors.iter())
                .map(|(a, c)| Stage {
                    x: byte_to_dpi(a[0]),
                    y: byte_to_dpi(a[1]),
                    color: [c[0], c[1], c[2]],
                    enabled: true,
                })
                .collect(),
        )
    });

    let dpi_effect = read_register(&link, DPI_EFFECT_ADDR, 6)
        .ok()
        .flatten()
        .map(|d| d[0] != 0);
    let polling = read_register(&link, POLLING_ADDR, 2)
        .ok()
        .flatten()
        .and_then(|d| match d[0] {
            0x01 => Some(1000),
            0x02 => Some(500),
            0x04 => Some(250),
            0x08 => Some(125),
            _ => None,
        });
    let buttons = read_buttons(&link);

    MouseState {
        present: true,
        mode,
        led,
        dpi,
        dpi_effect,
        polling,
        buttons,
    }
}

fn read_stage_table(link: &Link, base: u16) -> Option<Vec<[u8; 4]>> {
    let mut out = Vec::new();
    for (addr, len) in [(base, 8u8), (base + 8, 8), (base + 16, 4)] {
        let d = read_register(link, addr, len).ok().flatten()?;
        for i in (0..len as usize).step_by(4) {
            out.push([d[i], d[i + 1], d[i + 2], d[i + 3]]);
        }
    }
    Some(out)
}

fn read_buttons(link: &Link) -> Option<Vec<[u8; 3]>> {
    let mut out = Vec::new();
    for i in (0..BUTTON_COUNT).step_by(2) {
        let d = read_register(link, BUTTON_ADDR + 4 * i as u16, 8)
            .ok()
            .flatten()?;
        out.push([d[0], d[1], d[2]]);
        out.push([d[4], d[5], d[6]]);
    }
    Some(out)
}

pub fn set_led(
    api: &HidApi,
    effect: &str,
    r: u8,
    g: u8,
    b: u8,
    speed: u8,
    brightness: u8,
) -> Result<()> {
    let p = build_led(effect_id(effect), r, g, b, speed, brightness);
    apply_all(api, &[p])
}

pub fn set_dpi_stage(api: &HidApi, stage: usize, x: u32, y: u32) -> Result<()> {
    if !(1..=DPI_STAGES).contains(&stage) {
        return Err(DevError::Invalid("estágio deve ser 1..5".into()));
    }
    // Preserve the other stages: read the current bytes first (or fall back).
    let mut axes = read_link(api)
        .ok()
        .and_then(|l| read_stage_table(&l, DPI_ADDR))
        .unwrap_or_else(|| {
            (0..DPI_STAGES)
                .map(|_| [dpi_to_byte(x), dpi_to_byte(y), 0, 0])
                .collect()
        });
    axes[stage - 1] = [dpi_to_byte(x), dpi_to_byte(y), 0, 0];
    apply_all(api, &stage_table_packets(DPI_ADDR, &axes))
}

pub fn set_dpi_all(api: &HidApi, dpi: u32) -> Result<()> {
    let b = dpi_to_byte(dpi);
    let axes: Vec<[u8; 4]> = (0..DPI_STAGES).map(|_| [b, b, 0, 0]).collect();
    apply_all(api, &stage_table_packets(DPI_ADDR, &axes))
}

pub fn set_dpi_color(api: &HidApi, stage: usize, r: u8, g: u8, b: u8) -> Result<()> {
    if !(1..=DPI_STAGES).contains(&stage) {
        return Err(DevError::Invalid("estágio deve ser 1..5".into()));
    }
    let mut cols = read_link(api)
        .ok()
        .and_then(|l| read_stage_table(&l, DPI_COLOR_ADDR))
        .unwrap_or_else(|| (0..DPI_STAGES).map(|_| [r, g, b, 0]).collect());
    cols[stage - 1] = [r, g, b, 0];
    apply_all(api, &stage_table_packets(DPI_COLOR_ADDR, &cols))
}

fn stage_table_packets(base: u16, entries: &[[u8; 4]]) -> Vec<[u8; 17]> {
    let recs: Vec<Vec<u8>> = entries.iter().map(|e| record(&e[0..3])).collect();
    let mut d0 = recs[0].clone();
    d0.extend(&recs[1]);
    let mut d1 = recs[2].clone();
    d1.extend(&recs[3]);
    vec![
        build_write(base, &d0),
        build_write(base + 8, &d1),
        build_write(base + 16, &recs[4]),
    ]
}

pub fn set_dpi_effect(api: &HidApi, on: bool) -> Result<()> {
    let p = if on {
        // Preserve the two undecoded tail bytes when possible.
        let tail = read_link(api)
            .ok()
            .and_then(|l| read_register(&l, DPI_EFFECT_ADDR, 6).ok().flatten());
        let mut data = record(&[0x01]);
        if let Some(t) = tail {
            data.extend(record(&[t[2]]));
            data.extend(record(&[t[4]]));
        } else {
            data.extend(record(&[0x01]));
            data.extend(record(&[0x03]));
        }
        build_write(DPI_EFFECT_ADDR, &data)
    } else {
        build_write(DPI_EFFECT_ADDR, &record(&[0x00]))
    };
    apply_all(api, &[p])
}

pub fn set_polling(api: &HidApi, hz: u32) -> Result<()> {
    let code: u8 = if hz >= 1000 {
        0x01
    } else if hz >= 500 {
        0x02
    } else if hz >= 250 {
        0x04
    } else {
        0x08
    };
    apply_all(api, &[build_write(POLLING_ADDR, &record(&[code]))])
}

pub fn set_button(api: &HidApi, index: usize, action: Action) -> Result<()> {
    if index >= BUTTON_COUNT {
        return Err(DevError::Invalid("índice de botão deve ser 0..15".into()));
    }
    let mut table16 = [[0u8; 3]; BUTTON_COUNT];
    if let Some(cur) = read_link(api).ok().and_then(|l| read_buttons(&l)) {
        for (i, e) in cur.into_iter().enumerate() {
            table16[i] = e;
        }
    }
    table16[index] = action_to_record(&action);
    let mut packets = ext_packets_for(index, &action);
    packets.extend(build_buttons(&table16));
    apply_all(api, &packets)
}

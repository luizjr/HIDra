// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The table of devices HIDra knows how to talk to.
//!
//! **This is the one file you edit to add a device.** If your peripheral speaks
//! a [`Protocol`] that is already implemented, adding it is a single entry in
//! [`SUPPORTED`] — no other code changes. If it speaks something new, implement
//! the protocol in its own module and add a variant here.
//!
//! See `docs/ADDING-DEVICES.md` for how to find the numbers below.

use serde::Serialize;

/// What the device is, which decides the UI panel it gets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Keyboard,
    Mouse,
}

/// The wire dialect a device speaks. Each variant is implemented by one module.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Protocol {
    /// Compx/Areson mice (M711-PRO family): 17-byte feature reports on report
    /// id 0x08, every packet checksummed to 0x55. Implemented in `mouse.rs`.
    Compx17,
    /// SONiX SN32F keyboards (K586RGB-PRO family): 64-byte reports on report id
    /// 0x04, 16-bit sum checksum. Implemented in `keyboard.rs`.
    Sonix64,
}

/// How the device is attached — shown next to its name in the sidebar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Connection {
    /// Plugged in by cable.
    Wired,
    /// Reached through its own 2.4 GHz receiver (sleeps when idle).
    Receiver,
}

impl Connection {
    pub fn label(self) -> &'static str {
        match self {
            Connection::Wired => "wired",
            Connection::Receiver => "2.4g",
        }
    }
}

/// How much of this entry has actually been proven on real hardware.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Support {
    /// Someone ran HIDra against the device and everything applied.
    Verified,
    /// Added from a USB capture or a datasheet, never confirmed on hardware.
    Reported,
}

/// One USB product HIDra can configure.
///
/// A device that exposes the same product id over cable and over a receiver
/// gets one entry per id (see the M711-PRO below).
#[derive(Debug)]
pub struct Device {
    /// Brand printed on the box. Used for display only.
    pub vendor: &'static str,
    /// Marketing name plus internal model, e.g. `"Cobra Pro (M711-PRO)"`.
    pub model: &'static str,
    pub vid: u16,
    pub pid: u16,
    pub kind: Kind,
    pub protocol: Protocol,
    /// USB interface that carries the configuration endpoint.
    pub interface: i32,
    /// HID usage page of the collection we *write* configuration to. On Windows
    /// each top-level collection is a separate handle, so this is what picks the
    /// right one; on Linux/macOS the whole interface is one node and this is
    /// only a hint.
    pub config_usage_page: u16,
    /// Usage page of the collection the device *answers* on. Same as
    /// `config_usage_page` for most devices; the Compx mice reply on a
    /// different collection than the one they are written to.
    pub reply_usage_page: u16,
    pub connection: Connection,
    pub support: Support,
}

/// Every device HIDra recognises.
pub const SUPPORTED: &[Device] = &[
    // ---- Mice ----------------------------------------------------------
    Device {
        vendor: "Redragon",
        model: "Cobra Pro (M711-PRO)",
        vid: 0x25a7,
        pid: 0xfa07,
        kind: Kind::Mouse,
        protocol: Protocol::Compx17,
        interface: 1,
        config_usage_page: 0xff02,
        reply_usage_page: 0xff01,
        connection: Connection::Receiver,
        support: Support::Verified,
    },
    Device {
        vendor: "Redragon",
        model: "Cobra Pro (M711-PRO)",
        vid: 0x25a7,
        pid: 0xfa08,
        kind: Kind::Mouse,
        protocol: Protocol::Compx17,
        interface: 1,
        config_usage_page: 0xff02,
        reply_usage_page: 0xff01,
        connection: Connection::Wired,
        support: Support::Verified,
    },
    // ---- Keyboards -----------------------------------------------------
    Device {
        vendor: "Redragon",
        model: "Brahma Pro (K586RGB-PRO)",
        vid: 0x0c45,
        pid: 0x5004,
        kind: Kind::Keyboard,
        protocol: Protocol::Sonix64,
        interface: 1,
        config_usage_page: 0xff1c,
        reply_usage_page: 0xff1c,
        connection: Connection::Wired,
        support: Support::Verified,
    },
];

/// Every supported device of a kind, in table order.
pub fn of_kind(kind: Kind) -> impl Iterator<Item = &'static Device> {
    SUPPORTED.iter().filter(move |d| d.kind == kind)
}

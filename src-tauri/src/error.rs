// HIDra — open-source configurator for HID gaming peripherals.
// Copyright (C) 2026 HIDra contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevError {
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("dispositivo não encontrado — verifique o cabo/dongle e a regra udev")]
    NotFound,
    #[error("o dispositivo não respondeu (o mouse 2.4G dorme quando parado — mexa nele e tente de novo)")]
    NoResponse,
    #[error("{0}: este modelo usa um protocolo que ainda não foi implementado")]
    Unsupported(String),
    #[error("valor inválido: {0}")]
    Invalid(String),
}

impl serde::Serialize for DevError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DevError>;

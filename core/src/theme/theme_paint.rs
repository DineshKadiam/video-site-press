// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Degrees, Percent};

pub const DEFAULT_ACCENT_BRIGHTENING: u8 = 50;
pub const DEFAULT_ACCENT_CHROMA: u8 = 0;
pub const DEFAULT_ACCENT_HUE: u16 = 0;
pub const DEFAULT_BASE_CHROMA: u8 = 0;
pub const DEFAULT_BASE_HUE: u16 = 0;
pub const DEFAULT_CONTRAST: u8 = 0;
pub const DEFAULT_LAYERED_CHROMA: u8 = 0;

#[derive(Clone, Debug)]
pub struct ThemePaint {
    pub accent_brightening: Option<Percent>,
    pub accent_chroma: Option<Percent>,
    pub accent_hue: Option<Degrees>,
    pub base_chroma: Option<Percent>,
    pub base_hue: Option<Degrees>,
    pub contrast: Option<Percent>,
    pub layered_chroma: Option<Percent>
}

impl ThemePaint {
    pub fn accent_brightening(&self) -> u8 {
        self.accent_brightening
            .map(|accent_brightening| accent_brightening.value())
            .unwrap_or(DEFAULT_ACCENT_BRIGHTENING)
    }

    pub fn accent_chroma(&self) -> u8 {
        self.accent_chroma
            .map(|accent_chroma| accent_chroma.value())
            .unwrap_or(DEFAULT_ACCENT_CHROMA)
    }

    pub fn accent_hue(&self) -> u16 {
        self.accent_hue
            .map(|accent_hue| accent_hue.value())
            .unwrap_or(DEFAULT_ACCENT_HUE)
    }

    pub fn base_chroma(&self) -> u8 {
        self.base_chroma
            .map(|base_chroma| base_chroma.value())
            .unwrap_or(DEFAULT_BASE_CHROMA)
    }

    pub fn base_hue(&self) -> u16 {
        self.base_hue
            .map(|base_hue| base_hue.value())
            .unwrap_or(DEFAULT_BASE_HUE)
    }

    pub fn contrast(&self) -> u8 {
        self.contrast
            .map(|contrast| contrast.value())
            .unwrap_or(DEFAULT_CONTRAST)
    }

    pub fn layered_chroma(&self) -> u8 {
        self.layered_chroma
            .map(|layered_chroma| layered_chroma.value())
            .unwrap_or(DEFAULT_LAYERED_CHROMA)
    }
}

impl Default for ThemePaint {
    fn default() -> ThemePaint {
        ThemePaint {
            accent_brightening: None,
            accent_chroma: None,
            accent_hue: None,
            base_chroma: None,
            base_hue: None,
            contrast: None,
            layered_chroma: None
        }
    }
}
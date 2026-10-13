// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug)]
pub enum ThemeMode {
    Adaptive,
    Dark,
    Light
}

impl ThemeMode {
    pub fn key(&self) -> &'static str {
        match self {
            ThemeMode::Adaptive => "adaptive",
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light"
        }
    }
}

impl Default for ThemeMode {
    fn default() -> ThemeMode {
        ThemeMode::Adaptive
    }
}

impl TryFrom<&str> for ThemeMode {
    type Error = ();

    fn try_from(key: &str) -> Result<ThemeMode, ()> {
        match key {
            "adaptive" => Ok(ThemeMode::Adaptive),
            "dark" => Ok(ThemeMode::Dark),
            "light" => Ok(ThemeMode::Light),
            _ => Err(())
        }
    }
}

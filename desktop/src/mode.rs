// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_translations::Translations;

pub const MODES: [Mode; 3] = [
    Mode::Browser,
    Mode::Collaborative,
    Mode::Standalone
];

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Browser,
    Collaborative,
    Standalone
}

impl Mode {
    pub fn description(&self, translations: &Translations) -> &str {
        match self {
            Mode::Browser => translations.use_the_editor_in_your_browser,
            Mode::Collaborative => translations.share_the_editor_over_the_network,
            Mode::Standalone => translations.use_a_dedicated_editor_window
        }
    }

    pub fn from_key(key: &str) -> Mode {
        match key {
            "browser" => Mode::Browser,
            "collaborative" => Mode::Collaborative,
            "standalone" => Mode::Standalone,
            _ => Mode::Standalone
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Mode::Browser => hyper8_icons::BROWSER_MODE,
            Mode::Collaborative => hyper8_icons::COLLABORATIVE_MODE,
            Mode::Standalone => hyper8_icons::STANDALONE_MODE
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Mode::Browser => "browser",
            Mode::Collaborative => "collaborative",
            Mode::Standalone => "standalone"
        }
    }

    pub fn label(&self, translations: &Translations) -> &str {
        match self {
            Mode::Browser => translations.browser_mode,
            Mode::Collaborative => translations.collaborative_mode,
            Mode::Standalone => translations.standalone_mode
        }
    }
}

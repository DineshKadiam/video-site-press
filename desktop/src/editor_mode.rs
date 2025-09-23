// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_derive::{Deserialize, Serialize};

use hyper8_translations::Translations;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum EditorMode {
    BrowserBased,
    Collaborative,
    Standalone
}

impl EditorMode {
    pub fn description(&self, translations: &Translations) -> &str {
        match self {
            EditorMode::BrowserBased => translations.use_the_editor_in_your_browser,
            EditorMode::Collaborative => translations.share_the_editor_over_the_network,
            EditorMode::Standalone => translations.use_a_dedicated_editor_window
        }
    }

    pub fn from_key(key: &str) -> EditorMode {
        match key {
            "browser_based" => EditorMode::BrowserBased,
            "collaborative" => EditorMode::Collaborative,
            "standalone" => EditorMode::Standalone,
            _ => EditorMode::Standalone
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            EditorMode::BrowserBased => hyper8_icons::BROWSER_MODE,
            EditorMode::Collaborative => hyper8_icons::COLLABORATIVE_MODE,
            EditorMode::Standalone => hyper8_icons::STANDALONE_MODE
        }
    }

    pub fn key(&self) -> &str {
        match self {
            EditorMode::BrowserBased => "browser_based",
            EditorMode::Collaborative => "collaborative",
            EditorMode::Standalone => "standalone"
        }
    }

    pub fn label(&self, translations: &Translations) -> &str {
        match self {
            EditorMode::BrowserBased => translations.browser_based,
            EditorMode::Collaborative => translations.collaborative,
            EditorMode::Standalone => translations.standalone
        }
    }
}

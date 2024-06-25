// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use serde_derive::{Deserialize, Serialize};

use crate::Translations;

mod checkbox;
mod endpoints;
mod input;
mod select;
pub mod server;
mod textarea;
mod widgets;

use checkbox::Checkbox;
use input::Input;
use select::Select;
use textarea::Textarea;

#[derive(Debug, Deserialize, Serialize)]
pub struct EditorConfig {
    pub language_code: String,
    pub theme_key: String,
    pub tree_view_order: TreeViewOrder
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TreeViewOrder {
    Default,
    TitleAsc,
    TitleDesc
}

impl EditorConfig {
    pub fn new(
        language_code: &str,
        theme_key: &str,
        tree_view_order: TreeViewOrder
    ) -> EditorConfig {
        EditorConfig {
            language_code: language_code.into(),
            theme_key: theme_key.into(),
            tree_view_order: tree_view_order
        }
    }

    /// Persists editor configuration to a hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("hyper8");
            let config_file = app_dir.join("editor.bincode");

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&app_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }

    /// Retrieves editor configuration from a hyper8/ directory in the user
    /// system's config directory.
    pub fn retrieve() -> Option<EditorConfig> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_file = config_dir.join("hyper8").join("editor.bincode");

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(editor_config) = bincode::deserialize(&serialized) {
                    return Some(editor_config);
                }
            }
        }

        None
    }
}

impl TreeViewOrder {
    pub fn from_key(key: &str) -> TreeViewOrder {
        match key {
            "title_asc" => TreeViewOrder::TitleAsc,
            "title_desc" => TreeViewOrder::TitleDesc,
            _ => TreeViewOrder::Default
        }
    }

    pub fn key(&self, translations: &Translations) -> String {
        let t_title = translations.title;
        match self {
            TreeViewOrder::Default |
            TreeViewOrder::TitleAsc => format!("{t_title} ↑"),
            TreeViewOrder::TitleDesc => format!("{t_title} ↓")
        }
    }
}

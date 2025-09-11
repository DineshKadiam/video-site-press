// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::EditorState;

#[derive(Deserialize, Serialize)]
pub struct EditorConfig {
    pub language_code: String,
    pub mode_key: String,
    pub site_dirs: Vec<PathBuf>,
    pub theme_key: String,
    pub tree_view_order_key: String
}

impl EditorConfig {
    pub const DIR_NAME: &str = "hyper8";
    pub const FILE_NAME: &str = "editor.bincode";

    pub fn from_state(state: &EditorState) -> EditorConfig {
        let language_code = state.language.code.clone();
        let mode_key = state.mode.key().to_string();
        let theme_key = state.theme.key.to_string();
        let tree_view_order_key = state.tree_view_order.key().to_string();

        let site_dirs = state.site_bookmarks
            .iter()
            .map(|bookmark| bookmark.site_dir.clone())
            .collect();

        EditorConfig {
            language_code,
            mode_key,
            site_dirs,
            theme_key,
            tree_view_order_key
        }
    }

    /// Persists launcher configuration to a hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(user_config_dir) = dirs::config_dir() {
            let app_dir = user_config_dir.join(EditorConfig::DIR_NAME);
            let config_file = app_dir.join(EditorConfig::FILE_NAME);

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&app_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }

    /// Retrieves launcher configuration from a hyper8/ directory in the user
    /// system's config directory.
    pub fn retrieve() -> Option<EditorConfig> {
        if let Some(user_config_dir) = dirs::config_dir() {
            let config_file = user_config_dir
                .join(EditorConfig::DIR_NAME)
                .join(EditorConfig::FILE_NAME);

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(launcher_config) = bincode::deserialize(&serialized) {
                    return Some(launcher_config);
                }
            }
        }

        None
    }
}

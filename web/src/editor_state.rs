// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::{Arc, Mutex};

use serde_derive::{Deserialize, Serialize};

use hyper8_core::{Language, Theme};

use crate::TreeViewOrder;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EditorState {
    pub language: Language,
    pub theme: Theme,
    pub tree_view_order: TreeViewOrder
}

pub enum EditorStateConnection {
    InheritDefaults(EditorState),
    Shared(Arc<Mutex<EditorState>>)
}

impl EditorState {
    pub const CONFIG_DIR_NAME: &str = "hyper8";
    pub const CONFIG_FILE_NAME: &str = "editor.bincode";

    /// Retrieves the editor state from the hyper8/ directory in the user
    /// system's config directory (or initializes a default state).
    pub fn retrieve_or_initialize() -> EditorState {
        if let Some(user_config_dir) = dirs::config_dir() {
            let config_file = user_config_dir
                .join(EditorState::CONFIG_DIR_NAME)
                .join(EditorState::CONFIG_FILE_NAME);

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(editor_state) = bincode::deserialize(&serialized) {
                    return editor_state;
                }
            }
        }

        let language = Language::default();
        let theme = Theme::default();
        let tree_view_order = TreeViewOrder::Default;

        EditorState {
            language,
            theme,
            tree_view_order
        }
    }

    /// Persists launcher configuration to the hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(user_config_dir) = dirs::config_dir() {
            let config_dir = user_config_dir.join(EditorState::CONFIG_DIR_NAME);
            let config_file = config_dir.join(EditorState::CONFIG_FILE_NAME);

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&config_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }
}

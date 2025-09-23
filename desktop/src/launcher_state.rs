// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::{Arc, Mutex};

use serde_derive::{Deserialize, Serialize};

use hyper8_core::{Language, Theme};
use hyper8_translations::Translations;
use hyper8_web::EditorState;

use crate::{EditorMode, SiteBookmark};

#[derive(Debug, Deserialize, Serialize)]
pub struct LauncherState {
    pub editor_mode: EditorMode,
    pub editor_state: Arc<Mutex<EditorState>>,
    pub site_bookmarks: Vec<SiteBookmark>
}

impl LauncherState {
    pub const CONFIG_DIR_NAME: &str = "hyper8";
    pub const CONFIG_FILE_NAME: &str = "launcher.bincode";

    pub fn get_language_code(&self) -> String {
        self.editor_state.lock().unwrap().language.code.clone()
    }

    pub fn get_theme(&self) -> Theme {
        self.editor_state.lock().unwrap().theme.clone()
    }

    pub fn get_translations(&self) -> Translations {
        self.editor_state.lock().unwrap().language.translations.clone()
    }

    /// Persists launcher state to a hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(user_config_dir) = dirs::config_dir() {
            let config_dir = user_config_dir.join(LauncherState::CONFIG_DIR_NAME);
            let config_file = config_dir.join(LauncherState::CONFIG_FILE_NAME);

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&config_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }

    pub fn retrieve_or_initialize() -> LauncherState {
        if let Some(user_config_dir) = dirs::config_dir() {
            let config_file = user_config_dir
                .join(LauncherState::CONFIG_DIR_NAME)
                .join(LauncherState::CONFIG_FILE_NAME);

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(launcher_state) = bincode::deserialize(&serialized) {
                    return launcher_state;
                }
            }
        }

        let editor_mode = EditorMode::Standalone;
        let editor_state = Arc::new(Mutex::new(EditorState::retrieve_or_initialize()));
        let site_bookmarks = Vec::new();

        LauncherState {
            editor_mode,
            editor_state,
            site_bookmarks
        }
    }

    pub fn set_editor_mode(&mut self, editor_mode: EditorMode) {
        self.editor_mode = editor_mode;
        self.persist();
    }

    pub fn set_language(&mut self, language: Language) {
        let mut editor_state = self.editor_state.lock().unwrap();
        editor_state.language = language;
        editor_state.persist();
    }

    pub fn set_theme(&mut self, theme: Theme) {
        let mut editor_state = self.editor_state.lock().unwrap();
        editor_state.theme = theme;
        editor_state.persist();
    }
}

use std::fs;

use serde_derive::{Deserialize, Serialize};

mod checkbox;
mod endpoints;
mod input;
pub mod server;
mod widgets;

use checkbox::Checkbox;
use input::Input;

#[derive(Debug, Deserialize, Serialize)]
pub struct EditorConfig {
    pub language_code: String,
    pub theme_key: String
}

impl EditorConfig {
    pub fn new(language_code: &str, theme_key: &str) -> EditorConfig {
        EditorConfig {
            language_code: language_code.into(),
            theme_key: theme_key.into()
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


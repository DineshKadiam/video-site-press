use std::fs;

mod endpoints;
pub mod server;
mod widgets;

pub struct EditorConfig {
    pub language: String
}

/// Persists editor configuration to a hyper8/ directory in the user
/// system's config directory.
pub fn persist_config(language: &str) {
    if let Some(config_dir) = dirs::config_dir() {
        let app_dir = config_dir.join("hyper8");
        let language_file = app_dir.join("language");

        let _ = fs::create_dir(&app_dir);
        let _ = fs::write(&language_file, language);
    }
}

/// Retrieves editor configuration from a hyper8/ directory in the user
/// system's config directory.
pub fn retrieve_config() -> Option<EditorConfig> {
    if let Some(config_dir) = dirs::config_dir() {
        let language_file = config_dir.join("hyper8").join("language");

        if language_file.exists() {
            if let Ok(language) = fs::read_to_string(language_file) {
                return Some(
                    EditorConfig { language }
                );
            }
        }
    }

    None
}
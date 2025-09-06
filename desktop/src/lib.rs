// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Mutex;

use tauri::Manager;

mod commands;
mod launcher_config;
mod translations;

use launcher_config::LauncherConfig;
use translations::Translations;

pub struct LauncherState {
    config: LauncherConfig,
    translations: Translations
}

impl LauncherState {
    pub fn init() -> LauncherState {
        let config = LauncherConfig::retrieve()
            .unwrap_or_else(|| LauncherConfig::new("en", Vec::new()));

        let translations = Translations::from_code(&config.language_code);

        LauncherState {
            config,
            translations
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = LauncherState::init();
            app.manage(Mutex::new(state));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                commands::add_site,
                commands::change_language,
                commands::delete_site,
                commands::get_language,
                commands::render,
                commands::start_editor
            ]
        )
        .run(tauri::generate_context!())
        .unwrap();
}

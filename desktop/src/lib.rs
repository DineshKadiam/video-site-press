// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Mutex;

use tauri::{
    Manager,
    WebviewUrl,
    WebviewWindowBuilder
};

use hyper8_version::VERSION_WITHOUT_PATCH;

const EXIT_CODE_SUCCESS: i32 = 0;

mod commands;
mod editor;
mod launcher;
mod launcher_config;
mod launcher_state;
mod site_bookmark;

use launcher_config::LauncherConfig;
use launcher_state::LauncherState;
use site_bookmark::SiteBookmark;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let title = format!("Hyper 8 v{VERSION_WITHOUT_PATCH}");

            let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                    .center()
                    .decorations(false)
                    .inner_size(640.0, 640.0)
                    .max_inner_size(640.0, 640.0)
                    .min_inner_size(640.0, 640.0)
                    .title(&title)
                    .build()
                    .unwrap();

            let state = LauncherState::init();
            app.manage(Mutex::new(state));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                commands::change_language,
                commands::close,
                commands::create_new_site,
                commands::get_language,
                commands::get_theme,
                commands::open_bookmarked_site,
                commands::open_existing_site,
                commands::open_in_file_explorer,
                commands::remove_site_bookmark,
                commands::render
            ]
        )
        .run(tauri::generate_context!())
        .unwrap();
}

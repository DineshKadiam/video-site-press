// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    Manager,
    WebviewUrl,
    WebviewWindowBuilder
};

use hyper8_version::VERSION_ADAPTIVE_PATCH;

const EXIT_CODE_SUCCESS: i32 = 0;

mod commands;
mod editor;
mod editor_mode;
mod launcher_state;
mod site_bookmark;

use editor_mode::EditorMode;
use launcher_state::LauncherState;
use site_bookmark::SiteBookmark;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let title = format!("Hyper8 {VERSION_ADAPTIVE_PATCH}");

            let _ = WebviewWindowBuilder::new(app, "launcher", WebviewUrl::default())
                    .center()
                    .decorations(false)
                    .inner_size(640.0, 640.0)
                    .max_inner_size(640.0, 640.0)
                    .min_inner_size(640.0, 640.0)
                    .title(&title)
                    .build()
                    .unwrap();

            let state = LauncherState::retrieve_or_initialize();
            app.manage(Arc::new(Mutex::new(state)));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                commands::change_language::change_language,
                commands::change_mode::change_mode,
                commands::change_theme::change_theme,
                commands::close::close,
                commands::create_new_site::create_new_site,
                commands::get_language_code::get_language_code,
                commands::get_theme::get_theme,
                commands::open_bookmarked_site::open_bookmarked_site,
                commands::open_existing_site::open_existing_site,
                commands::open_in_file_manager::open_in_file_manager,
                commands::remove_site_bookmark::remove_site_bookmark,
                commands::render::render
            ]
        )
        .run(tauri::generate_context!())
        .unwrap();
}

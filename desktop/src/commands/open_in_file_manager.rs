// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

#[tauri::command]
pub fn open_in_file_manager(site_dir: &Path) {
    let _ = tauri_plugin_opener::open_path(site_dir, None::<&str>);
}

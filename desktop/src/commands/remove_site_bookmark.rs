// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use crate::LauncherState;

#[tauri::command]
pub fn remove_site_bookmark(
    app: AppHandle,
    site_dir: PathBuf,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    {
        let mut launcher_state = state.lock().unwrap();

        if let Some(index) = launcher_state.site_bookmarks
            .iter()
            .position(|bookmark| bookmark.site_dir == site_dir) {
            launcher_state.site_bookmarks.remove(index);
        }

        launcher_state.persist();
    }

    let _ = app.emit("update-body", ());
}

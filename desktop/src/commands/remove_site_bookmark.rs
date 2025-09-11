// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_web::EditorState;

#[tauri::command]
pub fn remove_site_bookmark(
    app: AppHandle,
    site_dir: PathBuf,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    {
        let mut state = state.lock().unwrap();

        if let Some(index) = state.site_bookmarks
            .iter()
            .position(|bookmark| bookmark.site_dir == site_dir) {
            state.site_bookmarks.remove(index);
        }

        state.persist();
    }

    let _ = app.emit("update-body", ());
}

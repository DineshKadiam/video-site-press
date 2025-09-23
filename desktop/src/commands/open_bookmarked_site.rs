// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    Manager,
    WebviewWindow
};

use crate::LauncherState;
use crate::editor;

/// Important: This command must be async (and/or the state retrieved inside
/// the function body rather than passed through the function arguments itself)
/// to avoid a deadlock on Windows that occurs when editor::start_editor later
/// calls WebviewWindowBuilder::new() with the passed AppHandle.
#[tauri::command]
pub async fn open_bookmarked_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    site_dir: PathBuf
) {
    let state = app.state::<Arc<Mutex<LauncherState>>>();

    // At this point the launcher could have been open for a long time
    // (and things on disk have changed quite a bit), therefore we check
    // again that the site_dir still exists.
    if site_dir.exists() {
        let editor_mode;
        let translations;

        {
            let mut launcher_state = state.lock().unwrap();

            let index = launcher_state.site_bookmarks
                .iter()
                .position(|bookmark| bookmark.site_dir == site_dir)
                .unwrap();

            // If the site is not at the top of our list of bookmarks,
            // put it there.
            if index > 0 {
                let bookmark = launcher_state.site_bookmarks.remove(index);
                launcher_state.site_bookmarks.insert(0, bookmark);
                launcher_state.persist();
            }

            editor_mode = launcher_state.editor_mode;
            translations = launcher_state.get_translations();
        }

        editor::start_editor(
            app,
            launcher_window,
            editor_mode,
            &site_dir,
            translations
        );
    } else {
        {
            let mut launcher_state = state.lock().unwrap();
            for bookmark in launcher_state.site_bookmarks.iter_mut() {
                if bookmark.site_dir == site_dir {
                    bookmark.found = false;
                }
            }
        }

         let _ = app.emit("update-body", ());
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State,
    WebviewWindow
};

use hyper8_web::EditorState;

use crate::editor;

#[tauri::command]
pub fn open_bookmarked_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    site_dir: &Path,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    // At this point the launcher could have been open for a long time
    // (and things on disk have changed quite a bit), therefore we check
    // again that the site_dir still exists.
    if site_dir.exists() {
        let mode;
        let translations;

        {
            let mut state_mut = state.lock().unwrap();

            let index = state_mut.site_bookmarks
                .iter()
                .position(|bookmark| bookmark.site_dir == site_dir)
                .unwrap();

            // If the site is not at the top of our list of bookmarks,
            // put it there.
            if index > 0 {
                let bookmark = state_mut.site_bookmarks.remove(index);
                state_mut.site_bookmarks.insert(0, bookmark);
                state_mut.persist();
            }

            mode = state_mut.mode;
            translations = state_mut.language.translations.clone();
        }

        editor::start_editor(
            app,
            launcher_window,
            mode,
            site_dir,
            translations
        );
    } else {
        {
            let mut state_mut = state.lock().unwrap();
            for bookmark in state_mut.site_bookmarks.iter_mut() {
                if bookmark.site_dir == site_dir {
                    bookmark.found = false;
                }
            }
        }

         let _ = app.emit("update-body", ());
    }
}

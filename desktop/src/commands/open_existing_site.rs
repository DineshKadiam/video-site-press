// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    Manager,
    State,
    WebviewWindow
};
use tauri_plugin_dialog::{DialogExt, FilePath};

use hyper8_web::{EditorState, SiteBookmark};

use crate::editor;

#[tauri::command]
pub fn open_existing_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    let state = state.lock().unwrap();

    app
        .dialog()
        .file()
        .set_title(state.language.translations.pick_an_empty_or_preexisting_site_directory)
        .pick_folder(move |result| {
            if let Some(FilePath::Path(site_dir)) = result {
                let bookmark = SiteBookmark::new(&site_dir);

                let state = app.state::<Arc<Mutex<EditorState>>>();

                let mode;
                let translations;

                {
                    let mut state_mut = state.lock().unwrap();

                    // If an already bookmarked site is opened, remove it form
                    // the bookmarks before we add it anew
                    if let Some(index) = state_mut.site_bookmarks
                        .iter()
                        .position(|bookmark| bookmark.site_dir == site_dir) {
                        state_mut.site_bookmarks.remove(index);
                    }

                    state_mut.site_bookmarks.insert(0, bookmark);

                    state_mut.persist();

                    mode = state_mut.mode;
                    translations = state_mut.language.translations.clone();
                }

                let _ = app.emit("update-body", ());

                editor::start_editor(
                    app,
                    launcher_window,
                    mode,
                    &site_dir,
                    translations
                );
            }
        });
}

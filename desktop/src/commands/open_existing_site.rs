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

use crate::editor;
use crate::{LauncherState, SiteBookmark};


#[tauri::command]
pub fn open_existing_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    let translations = state.lock().unwrap().get_translations();

    app
        .dialog()
        .file()
        .set_title(translations.pick_an_empty_or_preexisting_site_directory)
        .pick_folder(move |result| {
            if let Some(FilePath::Path(site_dir)) = result {
                let bookmark = SiteBookmark::new(&site_dir);

                let state = app.state::<Arc<Mutex<LauncherState>>>();

                let editor_mode;
                let translations;

                {
                    let mut launcher_state = state.lock().unwrap();

                    // If an already bookmarked site is opened, remove it form
                    // the bookmarks before we add it anew
                    if let Some(index) = launcher_state.site_bookmarks
                        .iter()
                        .position(|bookmark| bookmark.site_dir == site_dir) {
                        launcher_state.site_bookmarks.remove(index);
                    }

                    launcher_state.site_bookmarks.insert(0, bookmark);

                    launcher_state.persist();

                    editor_mode = launcher_state.editor_mode;
                    translations = launcher_state.get_translations();
                }

                let _ = app.emit("update-body", ());

                editor::start_editor(
                    app,
                    launcher_window,
                    editor_mode,
                    &site_dir,
                    translations
                );
            }
        });
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_core::Theme;
use hyper8_web::EditorState;

#[tauri::command]
pub fn change_theme(
    app: AppHandle,
    key: &str,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    let theme = Theme::from_key(key).unwrap();

    let mut state_mut = state.lock().unwrap();
    state_mut.theme = theme;
    state_mut.persist();

    // TODO: As an optimization, send along the rendered theme css already
    //       (as we do with language)
    let _ = app.emit("update-theme", ());
}

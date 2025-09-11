// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_core::Language;
use hyper8_web::EditorState;

#[tauri::command]
pub fn change_language(
    app: AppHandle,
    code: &str,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    let language = Language::from_code(code);

    let mut state_mut = state.lock().unwrap();
    state_mut.language = language;
    state_mut.persist();

    let _ = app.emit("update-body", ());
    let _ = app.emit("update-language", code);
}

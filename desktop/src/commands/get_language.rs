// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::State;

use hyper8_web::EditorState;

#[tauri::command]
pub fn get_language(
    state: State<'_, Arc<Mutex<EditorState>>>
) -> String {
    let state = state.lock().unwrap();
    state.language.code.to_string()
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_web::{EditorState, Mode};

#[tauri::command]
pub fn change_mode(
    app: AppHandle,
    key: &str,
    state: State<'_, Arc<Mutex<EditorState>>>
) {
    let mode = Mode::from_key(key);

    {
        let mut state_mut = state.lock().unwrap();
        state_mut.mode = mode;
        state_mut.persist();
    }

    let _ = app.emit("update-body", ());
}

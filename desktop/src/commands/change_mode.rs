// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use crate::{EditorMode, LauncherState};

#[tauri::command]
pub fn change_mode(
    app: AppHandle,
    key: &str,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    let editor_mode = EditorMode::from_key(key);

    {
        let mut launcher_state = state.lock().unwrap();
        launcher_state.set_editor_mode(editor_mode);
    }

    let _ = app.emit("update-body", ());
}

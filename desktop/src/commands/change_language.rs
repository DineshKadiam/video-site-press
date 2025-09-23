// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_core::Language;

use crate::LauncherState;

#[tauri::command]
pub fn change_language(
    app: AppHandle,
    code: &str,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    let language = Language::from_code(code);

    {
        let mut launcher_state = state.lock().unwrap();
        launcher_state.set_language(language);
    }

    let _ = app.emit("update-body", ());
    let _ = app.emit("update-language", code);
}

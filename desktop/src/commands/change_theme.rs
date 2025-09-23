// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle,
    Emitter,
    State
};

use hyper8_core::Theme;

use crate::LauncherState;

#[tauri::command]
pub fn change_theme(
    app: AppHandle,
    key: &str,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    let theme = Theme::from_key(key).unwrap();

    {
        let mut launcher_state = state.lock().unwrap();
        launcher_state.set_theme(theme);
    }

    // TODO: As an optimization, send along the rendered theme css already
    //       (as we do with language)
    let _ = app.emit("update-theme", ());
}

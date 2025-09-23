// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::LauncherState;

#[tauri::command]
pub fn get_language_code(
    state: State<'_, Arc<Mutex<LauncherState>>>
) -> String {
    state.lock().unwrap().get_language_code()
}

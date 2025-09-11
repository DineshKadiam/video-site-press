// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use tauri::AppHandle;

use crate::EXIT_CODE_SUCCESS;

#[tauri::command]
pub fn close(app: AppHandle) {
    app.exit(EXIT_CODE_SUCCESS);
}

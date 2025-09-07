// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::LauncherState;

#[derive(Deserialize, Serialize)]
pub struct LauncherConfig {
    pub language_code: String,
    pub site_dirs: Vec<PathBuf>,
}

impl LauncherConfig {
    pub fn from_state(state: &LauncherState) -> LauncherConfig {
        let language_code = state.language.code.clone();

        let site_dirs = state.site_bookmarks
            .iter()
            .map(|bookmark| bookmark.site_dir.clone())
            .collect();

        LauncherConfig {
            language_code,
            site_dirs
        }
    }

    /// Persists launcher configuration to a hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("hyper8");
            let config_file = app_dir.join("launcher.bincode");

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&app_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }

    /// Retrieves launcher configuration from a hyper8/ directory in the user
    /// system's config directory.
    pub fn retrieve() -> Option<LauncherConfig> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_file = config_dir.join("hyper8").join("launcher.bincode");

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(launcher_config) = bincode::deserialize(&serialized) {
                    return Some(launcher_config);
                }
            }
        }

        None
    }
}

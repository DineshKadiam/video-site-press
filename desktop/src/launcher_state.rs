// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::Language;

use crate::LauncherConfig;

pub struct LauncherState {
    pub config: LauncherConfig,
    pub language: Language
}

impl LauncherState {
    pub fn init() -> LauncherState {
        let config = LauncherConfig::retrieve()
            .unwrap_or_else(|| {
                let default_language = Language::default();
                LauncherConfig::new(default_language.code, Vec::new())
            });

        let language = Language::from_code(&config.language_code);

        LauncherState {
            config,
            language
        }
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::Language;

use crate::{LauncherConfig, SiteBookmark};

pub struct LauncherState {
    pub language: Language,
    pub site_bookmarks: Vec<SiteBookmark>
}

impl LauncherState {
    fn from_config(config: LauncherConfig) -> LauncherState {
        let language = Language::from_code(&config.language_code);

        let site_bookmarks = config.site_bookmark_paths
            .iter()
            .map(|path| SiteBookmark::new(path))
            .collect();

        LauncherState {
            language,
            site_bookmarks
        }
    }

    pub fn init() -> LauncherState {
        match LauncherConfig::retrieve() {
            Some(config) => LauncherState::from_config(config),
            None => {
                let language = Language::default();
                let site_bookmarks = Vec::new();

                LauncherState {
                    language,
                    site_bookmarks
                }
            }
        }
    }

    pub fn persist(&self) {
        let config = LauncherConfig::from_state(self);
        config.persist();
    }
}

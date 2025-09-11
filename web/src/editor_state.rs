// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::{Language, Theme};

use crate::{
    EditorConfig,
    Mode,
    SiteBookmark,
    TreeViewOrder
};

#[derive(Debug)]
pub struct EditorState {
    pub language: Language,
    pub mode: Mode,
    pub site_bookmarks: Vec<SiteBookmark>,
    pub theme: Theme,
    pub tree_view_order: TreeViewOrder
}

impl EditorState {
    fn from_config(config: EditorConfig) -> EditorState {
        let language = Language::from_code(&config.language_code);
        let mode = Mode::from_key(&config.mode_key);
        let theme = Theme::from_key(&config.theme_key)
            .unwrap_or(Theme::default());
        let tree_view_order = TreeViewOrder::from_key(&config.tree_view_order_key);

        let site_bookmarks = config.site_dirs
            .into_iter()
            .map(|site_dir| SiteBookmark::new(site_dir))
            .collect();

        EditorState {
            language,
            mode,
            site_bookmarks,
            theme,
            tree_view_order
        }
    }

    pub fn init() -> EditorState {
        match EditorConfig::retrieve() {
            Some(config) => EditorState::from_config(config),
            None => {
                let language = Language::default();
                let mode = Mode::Standalone;
                let site_bookmarks = Vec::new();
                let theme = Theme::default();
                let tree_view_order = TreeViewOrder::Default;

                EditorState {
                    language,
                    mode,
                    site_bookmarks,
                    theme,
                    tree_view_order
                }
            }
        }
    }

    pub fn persist(&self) {
        let config = EditorConfig::from_state(self);
        config.persist();
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use serde_derive::{Deserialize, Serialize};

use crate::{SitePath, Translations};

mod checkbox;
mod endpoints;
mod input;
mod routes;
mod select;
pub mod server;
mod textarea;
mod widgets;

use checkbox::Checkbox;
use input::Input;
use select::Select;
use textarea::Textarea;

#[derive(Debug, Deserialize, Serialize)]
pub struct EditorConfig {
    pub language_code: String,
    pub theme_key: String,
    pub tree_view_order: TreeViewOrder
}

/// Represents the page we are currently viewing, i.e. some element in the
/// site tree or an auxiliary page like the jobs overview.
pub enum Location<'a> {
    AuxiliaryPage,
    SiteTree {
        site_path: &'a SitePath,
        subpage: Subpage
    }
}

#[derive(PartialEq)]
pub enum Subpage {
    DangerousActions,
    Edit,
    Media,
    SiteSettings,
    Subtitles
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TreeViewOrder {
    Default,
    SiteOrder,
    TitleAsc,
    TitleDesc
}

impl EditorConfig {
    pub fn new(
        language_code: &str,
        theme_key: &str,
        tree_view_order: TreeViewOrder
    ) -> EditorConfig {
        EditorConfig {
            language_code: language_code.into(),
            theme_key: theme_key.into(),
            tree_view_order
        }
    }

    /// Persists editor configuration to a hyper8/ directory in the user
    /// system's config directory.
    pub fn persist(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("hyper8");
            let config_file = app_dir.join("editor.bincode");

            let config_serialized = bincode::serialize(&self).unwrap();

            let _ = fs::create_dir(&app_dir);
            let _ = fs::write(&config_file, config_serialized);
        }
    }

    /// Retrieves editor configuration from a hyper8/ directory in the user
    /// system's config directory.
    pub fn retrieve() -> Option<EditorConfig> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_file = config_dir.join("hyper8").join("editor.bincode");

            if let Ok(serialized) = fs::read(&config_file) {
                if let Ok(editor_config) = bincode::deserialize(&serialized) {
                    return Some(editor_config);
                }
            }
        }

        None
    }
}

impl<'a> Location<'a> {
    pub fn active_subpage(&self, site_path: &SitePath) -> Option<&Subpage> {
        match self {
            Location::AuxiliaryPage => None,
            Location::SiteTree { site_path: location_site_path, subpage } => {
                match site_path == *location_site_path {
                    true => Some(subpage),
                    false => None
                }
            }
        }
    }

    pub fn dangerous_actions(site_path: &SitePath) -> Location {
        Location::SiteTree {
            site_path,
            subpage: Subpage::DangerousActions
        }
    }

    pub fn edit(site_path: &SitePath) -> Location {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Edit
        }
    }

    pub fn intersects(&self, site_path: &SitePath) -> bool {
        match self {
            Location::AuxiliaryPage => false,
            Location::SiteTree { site_path: location_site_path, .. } => {
                location_site_path.starts_with(site_path)
            }
        }
    }

    pub fn media(site_path: &SitePath) -> Location {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Media
        }
    }

    pub fn site(site_path: &SitePath) -> Location {
        Location::SiteTree {
            site_path,
            subpage: Subpage::SiteSettings
        }
    }

    pub fn subtitles(site_path: &SitePath) -> Location {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Subtitles
        }
    }
}

impl TreeViewOrder {
    pub fn from_key(key: &str) -> TreeViewOrder {
        match key {
            "site_order" => TreeViewOrder::SiteOrder,
            "title_asc" => TreeViewOrder::TitleAsc,
            "title_desc" => TreeViewOrder::TitleDesc,
            _ => TreeViewOrder::Default
        }
    }

    pub fn key(&self, translations: &Translations) -> String {
        match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => translations.site_order.to_string(),
            TreeViewOrder::TitleAsc => format!("{} A-Z", translations.title),
            TreeViewOrder::TitleDesc => format!("{} Z-A", translations.title)
        }
    }
}

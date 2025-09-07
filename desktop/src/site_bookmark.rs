// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};

pub struct SiteBookmark {
    pub found: bool,
    pub live_url: Option<String>,
    pub site_dir: PathBuf,
    pub title: String
}

impl SiteBookmark {
    pub fn new(site_dir: PathBuf) -> SiteBookmark {
        let found = site_dir.exists();

        let live_url = SiteBookmark::probe_base_url(&site_dir);

        let title = SiteBookmark::probe_title(&site_dir)
            .unwrap_or_else(|| site_dir.file_name().unwrap().to_string_lossy().into());

        SiteBookmark {
            found,
            live_url,
            site_dir,
            title
        }
    }

    fn probe_base_url(site_dir: &Path) -> Option<String> {
        let site_manifest_path = site_dir.join("site.eno");

        let content = match fs::read_to_string(site_manifest_path) {
            Ok(content) => content,
            Err(_) => return None
        };

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(_) => return None
        };

        if let Some(element) = document.elements()
            .iter()
            .find(|element| element.key() == "base_url") {
            if let Ok(field) = element.as_field() {
                if let Ok(result) = field.value() {
                    if let Some(value) = result {
                        return Some(value.to_string());
                    }
                }
            }
        }

        None
    }

    fn probe_title(site_dir: &Path) -> Option<String> {
        let probe_title_from_manifest = |manifest_filename| -> Option<String> {
            let manifest_path = site_dir.join(manifest_filename);

            let content = match fs::read_to_string(manifest_path) {
                Ok(content) => content,
                Err(_) => return None
            };

            let document = match enolib::parse(&content) {
                Ok(document) => document,
                Err(_) => return None
            };

            if let Some(element) = document.elements()
                .iter()
                .find(|element| element.key() == "title") {
                if let Ok(field) = element.as_field() {
                    if let Ok(result) = field.value() {
                        if let Some(value) = result {
                            return Some(value.to_string());
                        }
                    }
                }
            }

            None
        };

        probe_title_from_manifest("collection.eno")
            .or_else(|| probe_title_from_manifest("playlist.eno"))
            .or_else(|| probe_title_from_manifest("video.eno"))
    }
}

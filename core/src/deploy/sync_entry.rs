// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

/// The `modified` field is parametrized to allow the FTP/SSH sync
/// implementations to use different types for modified timestamps.
#[derive(Debug)]
pub struct SyncEntry<M> {
    pub file_name: String,
    pub is_dir: bool,
    pub modified: M,
    pub path: PathBuf,
    pub size: u64
}

impl<M> SyncEntry<M> {
    pub fn type_based_retardation_index(&self) -> usize {
        match Path::new(&self.file_name).extension() {
            Some(extension) => {
                match &*extension.to_ascii_lowercase().to_string_lossy() {
                    "atom" | "rss" => 5,
                    "html" => 4,
                    "css" => 3,
                    "js" => 2,
                    "avif" | "jpeg" | "jpg" | "png" | "webp" => 1,
                    _ => 0
                }
            }
            None => 0
        }
    }
}

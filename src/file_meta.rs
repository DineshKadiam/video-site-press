// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct FileMeta {
    pub modified: SystemTime,
    /// File size in bytes
    pub size: u64
}

impl FileMeta {
    pub fn new(path: &Path) -> FileMeta {
        let metadata = fs::metadata(path).unwrap();

        FileMeta {
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: metadata.len()
        }
    }
}

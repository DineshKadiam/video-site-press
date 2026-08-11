// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use hyper8_util::hash;

use crate::{Context, FileMeta, SitePath, AudioMetaCached};

/// Stores metadata that is specific to an audio file
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioMeta {
    pub codec: Option<String>,
    /// Duration in seconds
    pub duration: f32
}

impl AudioMeta {
    /// Increment this cache serialization key when:
    /// - The data layout of [AudioMeta] changes
    /// - The way we compute [AudioMeta] changes
    ///
    /// This automatically lets the cache discard audio meta manifests that
    /// hold incorrect, incompatible or outdated data.
    pub const CACHE_SERIALIZATION_KEY: &'static str = "audio_meta1";

    pub fn deserialize_cached(path: &Path) -> Option<AudioMetaCached> {
        match fs::read(path) {
            Ok(bytes) => bincode::deserialize::<AudioMetaCached>(&bytes).ok(),
            Err(_) => None
        }
    }

    pub fn new(
        codec: Option<String>,
        duration: f32
    ) -> AudioMeta {
        AudioMeta {
            codec,
            duration
        }
    }

    pub fn write_cache_manifest(
        &self,
        context: &Context,
        file_meta: &FileMeta,
        file_name: &str,
        site_path: &SitePath
    ) {
        let version_path = format!("{}{}", site_path.normalized(), file_name);

        let cache_item = AudioMetaCached {
            audio_meta: self.clone(),
            file_meta: file_meta.clone(),
            path: version_path.clone()
        };

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let cache_item_stem = hash(&version_path);
        let cache_item_key = AudioMeta::CACHE_SERIALIZATION_KEY;

        let cache_item_file_name = format!("{cache_item_stem}.{cache_item_key}.bincode");
        let cache_item_path = context.cache_dir.join(cache_item_file_name);

        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::{Context, FileMeta, SitePath, VideoMetaCached};
use crate::util::hash;

/// Stores metadata that is specific to a video file
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VideoMeta {
    pub audio_codec: Option<String>,
    /// Duration in seconds
    pub duration: f32,
    pub height: usize,
    pub video_codec: Option<String>,
    pub video_frame_rate: f32,
    pub width: usize
}

impl VideoMeta {
    /// Increment this cache serialization key when:
    /// - The data layout of [VideoMeta] changes
    /// - The way we compute [VideoMeta] changes
    ///
    /// This automatically lets the cache discard video meta manifests that
    /// hold incorrect, incompatible or outdated data.
    pub const CACHE_SERIALIZATION_KEY: &'static str = "video_meta1";

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn deserialize_cached(path: &Path) -> Option<VideoMetaCached> {
        match fs::read(path) {
            Ok(bytes) => bincode::deserialize::<VideoMetaCached>(&bytes).ok(),
            Err(_) => None
        }
    }

    pub fn new(
        audio_codec: Option<String>,
        duration: f32,
        height: usize,
        video_codec: Option<String>,
        video_frame_rate: f32,
        width: usize
    ) -> VideoMeta {
        VideoMeta {
            audio_codec,
            duration,
            height,
            video_codec,
            video_frame_rate,
            width
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

        let cache_item = VideoMetaCached {
            file_meta: file_meta.clone(),
            path: version_path.clone(),
            video_meta: self.clone()
        };

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let cache_item_stem = hash(&version_path);
        let cache_item_key = VideoMeta::CACHE_SERIALIZATION_KEY;

        let cache_item_file_name = format!("{cache_item_stem}.{cache_item_key}.bincode");
        let cache_item_path = context.cache_dir.join(cache_item_file_name);

        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}

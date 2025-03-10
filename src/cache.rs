// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::{
    FileMeta,
    PosterAssets,
    SitePath,
    VideoMeta
};
use crate::util;

/// This is the name of an empty file created by hyper8 in the root
/// of the cache directory. When the cache layout (or critical implementation
/// details) change, this name can be updated, prompting cache purge and rebuild
/// for site operators picking up the new version of hyper8.
///
/// History:
/// 1->2 (2024-04-27): VideoMeta fields changed, but bincode did not pick it up, resulting in faulty decodes
/// 2->3 (2024-05-4): VideoMeta fields changed, likely faulty decodes from bincode
const CACHE_VERSION_MARKER: &str = "CACHE_VERSION_MARKER_3";

pub struct Cache {
    pub poster_assets: Vec<PosterAssetsCached>,
    pub video_meta: Vec<VideoMetaCached>
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CacheItem {
    PosterAssets(PosterAssetsCached),
    VideoMeta(VideoMetaCached)
}

/// Caches differently sized, preprocessed versions of a poster.
/// In order to reassociate cached poster assets with
/// its source image, this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Debug, Deserialize, Serialize)]
pub struct PosterAssetsCached {
    pub assets: PosterAssets,
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk. 
    pub path: String
}

/// Caches metadata for a video that was (somewhat expensively) computed
/// using ffprobe. In order to reassociate a cached metadata entry with
/// its video(s), this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoMetaCached {
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk. 
    pub path: String,
    pub video_meta: VideoMeta
}

impl Cache {
    pub fn fetch_poster_assets(
        &self,
        file_meta: &FileMeta,
        file_name: &str,
        poster_aspect: f32,
        site_path: &SitePath
    ) -> Option<PosterAssets> {
        let poster_path = format!("{}{file_name}", site_path.normalized());

        self.poster_assets
            .iter()
            .find(|cached|
                cached.path == poster_path &&
                cached.file_meta == *file_meta &&
                cached.assets.poster_aspect == poster_aspect
            )
            .map(|cached| cached.assets.clone())
    }

    /// Scans the cache dir and initializes a HashMap that tracks
    /// all filenames, mapping them to a usage count (initialized to 0).
    /// The asset cache version marker is explicitly excluded from this.
    fn fill_registry(cache_dir: &Path) -> HashMap<String, usize> {
        let mut artifact_registry: HashMap<String, usize> = HashMap::new();

        if let Ok(dir_entries) = cache_dir.read_dir() {
            for dir_entry_result in dir_entries {
                if let Ok(dir_entry) = dir_entry_result {
                    if dir_entry
                        .file_type()
                        .map(|file_type| file_type.is_file())
                        .unwrap_or(false) {
                        let file_name = dir_entry.file_name().to_str().unwrap().to_string();

                        if !file_name.ends_with(".bincode") && file_name != CACHE_VERSION_MARKER {
                            artifact_registry.insert(file_name, 0);
                        }
                    }
                }
            }
        }

        artifact_registry
    }

    fn new() -> Cache {
        Cache {
            poster_assets: Vec::new(),
            video_meta: Vec::new()
        }
    }

    fn remove_orphaned(artifact_registry: HashMap<String, usize>, cache_dir: &Path) {
        for (filename, usage_counter) in artifact_registry {
            if usage_counter == 0 {
                eprintln!(
                    "Removing orphaned cache artifact ({}) - it was probably created with a different version of hyper8 or stems from a cancelled background job.",
                    filename
                );
                let _ = fs::remove_file(cache_dir.join(filename));
            }
        }
    }

    pub fn retrieve(cache_dir: &Path) -> Cache {
        let mut cache = Cache::new();

        let version_marker_file = cache_dir.join(CACHE_VERSION_MARKER);

        if !version_marker_file.exists() {
            if cache_dir.exists() {
                eprintln!("Existing cache data is in an incompatible format (= from an older hyper8 version), the cache will be purged and regenerated.");
                util::ensure_empty_dir(cache_dir);
            } else {
                util::ensure_dir(cache_dir);
            }
            fs::write(version_marker_file, "").unwrap();
        }

        // We register all files found in the cache (except manifests =
        // bincode files) here before we read the cache manifests. Files
        // referenced in the manifests that do not appear in the registry
        // mean that the cache entry is corrupt (we then remove it). The
        // other way around, every time we find a file in the registry we
        // increase its usage count (the value in the HashMap). At the end of
        // the cache retrieval process we know that all files in the registry
        // with a usage count of 0 are orphaned and can therefore be
        // removed.
        let mut artifact_registry = Cache::fill_registry(cache_dir);

        if let Ok(dir_entries) = cache_dir.read_dir() {
            for dir_entry_result in dir_entries {
                if let Ok(dir_entry) = dir_entry_result {
                    if !dir_entry.file_type().is_ok_and(|file_type| file_type.is_file()) {
                        continue
                    }

                    let filename = dir_entry.file_name().to_string_lossy().to_string();

                    if !filename.ends_with(".bincode") { continue }

                    let serialized = fs::read(dir_entry.path()).unwrap();

                    if let Ok(cache_item) = bincode::deserialize(&serialized) {
                        match cache_item {
                            CacheItem::PosterAssets(poster_assets) => {
                                let all_assets = poster_assets.assets.all();
                                for asset in &all_assets {
                                    if let Some(usage_counter) = artifact_registry.get_mut(&asset.file_name) {
                                        *usage_counter += 1;
                                    } else {
                                        // If a single asset is in a corrupt state (cached file missing)
                                        // we delete all other assets and remove the cache entry altogether.

                                        for asset_to_delete in &all_assets {
                                            let asset_path = cache_dir.join(&asset_to_delete.file_name);
                                            let _ = fs::remove_file(&asset_path);
                                        }

                                        eprintln!("Removing corrupted cache item {filename}");
                                        let _ = fs::remove_file(dir_entry.path());
                                        break;
                                    }
                                }

                                cache.poster_assets.push(poster_assets);
                            }
                            CacheItem::VideoMeta(video_meta) => {
                                cache.video_meta.push(video_meta);
                            }
                        }
                    } else {
                        eprintln!("Removing incompatible cache item {filename}");
                        let _ = fs::remove_file(dir_entry.path());
                    }
                }
            }
        }

        Cache::remove_orphaned(artifact_registry, cache_dir);

        cache
    }
}

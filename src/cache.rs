// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use log::info;
use serde_derive::{Deserialize, Serialize};

use crate::{
    AudioMeta,
    FileMeta,
    PosterAssets,
    PosterMeta,
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
/// 2->3 (2024-05-04): VideoMeta fields changed, likely faulty decodes from bincode
/// 3->4 (2025-07-12): Our ffprobe routine now detects rotation side data, we invalidate the cache to ensure
///                    previously (wrongly) computed height/width is recomputed to take this into account.
/// 4->5 (2025-07-19): The base CacheItem enum gained another variant (PosterSource)
/// "CACHE_VERSION_MARKER_5" -> "cache1.marker" (2025-07-19):
///     We refactored the entire cache logic to discretely de/serialize and
///     consider/discard manifests of each content type by their extension
///     (cache serialization key). We can still reset the entire cache but it
///     will be needed much less from here on out.
const CACHE_VERSION_MARKER: &str = "cache1.marker";

/// Caches metadata for an audio file that was (somewhat expensively) computed
/// using ffprobe. In order to reassociate a cached metadata entry with
/// its audio file(s), this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Debug, Deserialize, Serialize)]
pub struct AudioMetaCached {
    pub audio_meta: AudioMeta,
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk.
    pub path: String,
}

pub struct Cache {
    pub audio_meta: Vec<AudioMetaCached>,
    pub poster_assets: Vec<PosterAssetsCached>,
    pub poster_sources: Vec<PosterSourceCached>,
    pub video_meta: Vec<VideoMetaCached>
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

/// Caches poster image metadata such as the dimensions of the source image
/// and a reduced set of predominant colors found in the image (these can
/// be used for very small and/or very fast low fidelity display).
/// In order to reassociate a cached poster with
/// its source image, this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Debug, Deserialize, Serialize)]
pub struct PosterSourceCached {
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk.
    pub path: String,
    pub poster_meta: PosterMeta
}

/// Tracks all assets (images and such) and manifests (serialized bincode)
/// present in the cache directory during the cache retrieval process.
pub struct Registry {
    assets: HashMap<String, bool>,
    manifests: Vec<String>
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

    pub fn fetch_poster_source(
        &self,
        file_meta: &FileMeta,
        file_name: &str,
        site_path: &SitePath
    ) -> Option<PosterMeta> {
        let poster_path = format!("{}{file_name}", site_path.normalized());

        self.poster_sources
            .iter()
            .find(|cached|
                cached.path == poster_path &&
                cached.file_meta == *file_meta
            )
            .map(|cached| cached.poster_meta.clone())
    }

    /// Scans the cache dir and initializes:
    /// 1. A HashMap that tracks all assets, mapping them to a usage count (initialized to 0)
    /// 2. A Vec that stores all manifests
    /// The asset cache version marker is explicitly excluded from this, not being
    /// included in either of the two.
    fn fill_registry(cache_dir: &Path) -> Registry {
        let mut registry = Registry::new();

        if let Ok(dir_entries) = cache_dir.read_dir() {
            for dir_entry_result in dir_entries {
                if let Ok(dir_entry) = dir_entry_result {
                    if dir_entry
                        .file_type()
                        .map(|file_type| file_type.is_file())
                        .unwrap_or(false) {
                        let file_name = dir_entry.file_name().to_str().unwrap().to_string();

                        if file_name.ends_with(".bincode") {
                            registry.manifests.push(file_name);
                        } else if file_name != CACHE_VERSION_MARKER {
                            registry.assets.insert(file_name, false);
                        }
                    }
                }
            }
        }

        registry
    }

    fn new() -> Cache {
        Cache {
            audio_meta: Vec::new(),
            poster_assets: Vec::new(),
            poster_sources: Vec::new(),
            video_meta: Vec::new()
        }
    }

    fn remove_orphaned_assets(cache_dir: &Path, registry: Registry) {
        for (filename, used) in registry.assets {
            if !used {
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

        // We register all assets and manifests found in the cache here before
        // we begin. Assets referenced in the manifests that do not appear in
        // the registry mean that the cache entry is corrupt (we then remove
        // it). The other way around, every time we find an asset in the
        // registry we mark it as used (inside the value in the HashMap). At
        // the end of the cache retrieval process we know that all files in
        // the registry we did not tag as used are orphaned and can therefore
        // be removed.
        let mut registry = Cache::fill_registry(cache_dir);

        for file_name in &registry.manifests {
            if file_name.ends_with(&format!(".{}.bincode", AudioMeta::CACHE_SERIALIZATION_KEY)) {
                cache.retrieve_audio_meta(cache_dir, &file_name);
            } else if file_name.ends_with(&format!(".{}.bincode", PosterAssets::CACHE_SERIALIZATION_KEY)) {
                cache.retrieve_poster_assets(&mut registry.assets, cache_dir, &file_name);
            } else if file_name.ends_with(&format!(".{}.bincode", PosterMeta::CACHE_SERIALIZATION_KEY)) {
                cache.retrieve_poster_source(cache_dir, &file_name);
            } else if file_name.ends_with(&format!(".{}.bincode", VideoMeta::CACHE_SERIALIZATION_KEY)) {
                cache.retrieve_video_meta(cache_dir, &file_name);
            } else {
                info!(
                    "Removing incompatible cache manifest {file_name} - it was probably created with a different version of Hyper 8."
                );
                let _ = fs::remove_file(cache_dir.join(&file_name));
            }
        }

        Cache::remove_orphaned_assets(cache_dir, registry);

        cache
    }

    fn retrieve_audio_meta(
        &mut self,
        cache_dir: &Path,
        file_name: &str
    ) {
        let manifest_path = cache_dir.join(file_name);

        if let Some(audio_meta) = AudioMeta::deserialize_cached(&manifest_path) {
            self.audio_meta.push(audio_meta);
        } else {
            info!("Removing incompatible audio meta cache manifest ({file_name}) - it was probably created with a different version of Hyper 8.");
            let _ = fs::remove_file(&manifest_path);
        }
    }

    fn retrieve_poster_assets(
        &mut self,
        assets: &mut HashMap<String, bool>,
        cache_dir: &Path,
        file_name: &str
    ) {
        let manifest_path = cache_dir.join(file_name);

        if let Some(poster_assets) = PosterAssets::deserialize_cached(&manifest_path) {
            let all_assets = poster_assets.assets.all();
            for asset in &all_assets {
                if let Some(used) = assets.get_mut(&asset.file_name) {
                    *used = true;
                } else {
                    // If a single asset is in a corrupt state (cached file missing)
                    // we delete all other assets and remove the cache entry altogether.

                    for asset_to_delete in &all_assets {
                        let asset_path = cache_dir.join(&asset_to_delete.file_name);
                        let _ = fs::remove_file(&asset_path);
                    }

                    eprintln!("Removing corrupted poster assets cache item {}", manifest_path.display());
                    let _ = fs::remove_file(&manifest_path);
                    return;
                }
            }

            self.poster_assets.push(poster_assets);
        } else {
            info!(
                "Removing incompatible poster assets cache manifest ({file_name}) - it was probably created with a different version of Hyper 8."
            );
            let _ = fs::remove_file(&manifest_path);
        }
    }

    fn retrieve_poster_source(
        &mut self,
        cache_dir: &Path,
        file_name: &str
    ) {
        let manifest_path = cache_dir.join(file_name);

        if let Some(poster_source) = PosterMeta::deserialize_cached(&manifest_path) {
            self.poster_sources.push(poster_source);
        } else {
            info!("Removing incompatible poster source cache manifest ({file_name}) - it was probably created with a different version of Hyper 8.");
            let _ = fs::remove_file(&manifest_path);
        }
    }

    fn retrieve_video_meta(
        &mut self,
        cache_dir: &Path,
        file_name: &str
    ) {
        let manifest_path = cache_dir.join(file_name);

        if let Some(video_meta) = VideoMeta::deserialize_cached(&manifest_path) {
            self.video_meta.push(video_meta);
        } else {
            info!("Removing incompatible video meta cache manifest ({file_name}) - it was probably created with a different version of Hyper 8.");
            let _ = fs::remove_file(&manifest_path);
        }
    }
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            assets: HashMap::new(),
            manifests: Vec::new()
        }
    }
}

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde_derive::{Deserialize, Serialize};

use crate::{PosterAssets, VideoMeta};

pub struct Cache {
	pub playlists: Vec<PlaylistInternals>,
	pub poster_assets: Vec<PosterAssetsCached>,
	pub video_meta: Vec<VideoMetaCached>,
	pub videos: Vec<VideoInternals>
}

#[derive(Deserialize, Serialize)]
pub enum CacheItem {
	Playlist(PlaylistInternals),
	Video(VideoInternals),
	PosterAssets(PosterAssetsCached),
	VideoMeta(VideoMetaCached)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileMeta {
    pub modified: SystemTime,
    /// File size in bytes
    pub size: u64
}

/// Caches differently sized, preprocessed versions of a poster.
/// In order to reassociate cached poster assets with
/// its source image, this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Deserialize, Serialize)]
pub struct PosterAssetsCached {
    pub assets: PosterAssets,
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk. 
    pub path: String
}

/// We don't cache any actual data, for now.
#[derive(Deserialize, Serialize)]
pub struct PlaylistInternals {
	pub path: String
}

#[derive(Deserialize, Serialize)]
pub struct VideoInternals {
	pub id: String,
	pub slug: String,
	pub source: Option<String>
}

/// Caches metadata for a video that was (somewhat expensively) computed
/// using ffprobe. In order to reassociate a cached metadata entry with
/// its video(s), this is the strategy:
/// - If the path, modified and size match, we assume it's the same file
/// - For now there is no other strategy
#[derive(Deserialize, Serialize)]
pub struct VideoMetaCached {
    pub file_meta: FileMeta,
    /// The path is relative to the site_dir. This ensures
    /// that we can correctly re-associate files on each build, even
    /// if the site dir directory moves around on disk. 
    pub path: String,
    pub video_meta: VideoMeta
}

impl Cache {
	fn new() -> Cache {
		Cache {
			playlists: Vec::new(),
			poster_assets: Vec::new(),
			video_meta: Vec::new(),
			videos: Vec::new()
		}
	}

	pub fn retrieve(cache_dir: &Path) -> Cache {
		let mut cache = Cache::new();

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
	                		CacheItem::Playlist(playlist_internals) => {
	                			cache.playlists.push(playlist_internals);
	                		}
	                		CacheItem::PosterAssets(poster_assets) => {
	                			cache.poster_assets.push(poster_assets);
	                		}
	                		CacheItem::Video(video_internals) => {
	                			cache.videos.push(video_internals);
	                		}
	                		CacheItem::VideoMeta(video_meta) => {
	                			cache.video_meta.push(video_meta);
	                		}
	                	}
	                } else {
	                	eprintln!("Removing incompatible cache item {filename}");
	                	fs::remove_file(dir_entry.path()).unwrap();
	                }
	            }
	        }
	    }

	    cache
	}
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

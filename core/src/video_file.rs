// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::{
    Context,
    FileMeta,
    MediaUsage,
    SortDirection,
    Video,
    VideoMeta
};

/// Represents a specific "version" and physical file for a video.
/// For instance the 480p and 720p versions of a video are both one
/// separate [VideoFile].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VideoFile {
    pub explicitly_defined: bool,
    pub file_meta: FileMeta,
    pub file_name: String,
    pub usage: MediaUsage,
    pub video_meta: Option<VideoMeta>
}

impl VideoFile {
    pub const SUPPORTED_EXTENSIONS: [&str; 5] = ["m4v", "mp4", "mkv", "mov", "webm"];

    /// Determine the ordering between two [VideoFile] based on dimensions
    /// (height), frame rate and size (in that priority). As fallback rely on
    /// presence of metadata and raw filesize to determine some ordering.
    pub fn cmp_fidelity(
        a: &VideoFile,
        b: &VideoFile,
        direction: SortDirection
    ) -> Ordering {
        let ordering = match (&a.video_meta, &b.video_meta) {
            (Some(a_video_meta), Some(b_video_meta)) => {
                if a_video_meta.height != b_video_meta.height {
                    a_video_meta.height.cmp(&b_video_meta.height)
                } else if a_video_meta.video_frame_rate != b_video_meta.video_frame_rate {
                    a_video_meta.video_frame_rate.partial_cmp(&b_video_meta.video_frame_rate)
                        .unwrap_or_else(|| a.file_meta.size.cmp(&b.file_meta.size))
                } else {
                    a.file_meta.size.cmp(&b.file_meta.size)
                }
            }
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.file_meta.size.cmp(&b.file_meta.size)
        };

        direction.orient(ordering)
    }

    pub fn download_restricted(&self) -> bool {
        match self.usage {
            MediaUsage::Download |
            MediaUsage::Unrestricted => false,
            MediaUsage::Offline |
            MediaUsage::Streaming => true
        }
    }

    pub fn mime_type(&self) -> &'static str {
        let (_stem, extension) = self.file_name.rsplit_once('.').unwrap();

        // This mapping must be kept up-to-date with
        // [SUPPORTED_EXTENSIONS], otherwise it's a full on-crash somewhere
        // (but at least something that is quickly and easily discovered).
        match extension.to_ascii_lowercase().as_str() {
            "m4v" |
            "mp4" => "video/mp4",
            "mkv" => "video/x-matroska",
            "mov" => "video/quicktime",
            "webm" => "video/webm",
            _ => unimplemented!()
        }
    }

    /// Sometimes we intialize a VideoFile right after we encoded it
    /// ourselves, in that case we do not perform a video meta cache lookup
    pub fn new(
        context: &Context,
        path: &Path,
        query_cached_video_meta: bool,
        video: &Video
    ) -> VideoFile {
        let file_meta = FileMeta::new(&path);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let video_meta = if query_cached_video_meta {
            let version_path = format!("{}{}", &video.site_path.normalized(), file_name);

            context.cache.lock().unwrap().video_meta
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
                .map(|cached| cached.video_meta.clone())
        } else {
            None
        };

        VideoFile {
            explicitly_defined: false,
            file_meta,
            file_name,
            usage: MediaUsage::Unrestricted,
            video_meta
        }
    }

    pub fn new_from_manifest(
        context: &Context,
        path: &Path,
        usage: MediaUsage,
        video: &Video
    ) -> VideoFile {
        let file_meta = FileMeta::new(path);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let version_path = format!("{}{}", &video.site_path.normalized(), file_name);
        let video_meta = context.cache.lock().unwrap().video_meta
            .iter()
            .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
            .map(|cached| cached.video_meta.clone());

        VideoFile {
            explicitly_defined: true,
            file_meta,
            file_name,
            usage,
            video_meta
        }
    }

    /// An video file is considered offline if its usage has been restricted
    /// to offline
    pub fn offline(&self) -> bool {
        match self.usage {
            MediaUsage::Download |
            MediaUsage::Streaming |
            MediaUsage::Unrestricted => false,
            MediaUsage::Offline => true
        }
    }

    /// Video files are automatically registered and associated with its
    /// video, and usually do not need to be mentioned in the manifest
    /// (then this method returns true). However, if they were already
    /// explicitly defined in the manifest when we initialized the [Video], or
    /// if some non-default configuration value was set on the video file, we
    /// cannot skip the manifest entry (then this method returns false).
    pub fn skip_manifest_entry(&self) -> bool {
        !self.explicitly_defined && self.usage == MediaUsage::Unrestricted
    }

    pub fn streaming_restricted(&self) -> bool {
        match self.usage {
            MediaUsage::Download |
            MediaUsage::Offline => true,
            MediaUsage::Streaming |
            MediaUsage::Unrestricted => false
        }
    }

    pub fn supported_extension(filename: &str) -> bool {
        Path::new(filename).extension().is_some_and(|extension| {
            VideoFile::SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported_extension| {
                    extension.eq_ignore_ascii_case(supported_extension)
                })
        })
    }

    pub fn vertical_resolution_label(&self) -> String {
        match &self.video_meta {
            Some(video_meta) => format!("{}p", video_meta.height),
            None => String::new()
        }
    }

    pub fn video_meta_unchecked(&self) -> &VideoMeta {
        self.video_meta.as_ref().unwrap()
    }
}

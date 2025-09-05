// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

use serde_derive::{Deserialize, Serialize};

use crate::{
    FileMeta,
    SortDirection,
    VideoMeta
};

/// Represents a specific "version" and physical file for a video.
/// For instance the 480p and 720p versions of a video are both one
/// separate [VideoFile].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VideoFile {
    pub file_meta: FileMeta,
    pub file_name: String,
    pub video_meta: Option<VideoMeta>
}

impl VideoFile {
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

    pub fn mime_type(&self) -> &'static str {
        let (_stem, extension) = self.file_name.rsplit_once('.').unwrap();

        // This mapping must be kept up-to-date with what we accept in our
        // [supported_video_extension] helper function, otherwise it's a full
        // on-crash (but at least something that is quickly and easily
        // discovered).
        match extension {
            "m4v" |
            "mp4" => "video/mp4",
            "mkv" => "video/x-matroska",
            "mov" => "video/quicktime",
            "webm" => "video/webm",
            _ => unimplemented!()
        }
    }

    pub fn new(
        file_meta: FileMeta,
        file_name: String,
        video_meta: Option<VideoMeta>
    ) -> VideoFile {
        VideoFile {
            file_meta,
            file_name,
            video_meta
        }
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
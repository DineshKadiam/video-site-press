// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use crate::{
    AudioFormat,
    Collection,
    Playlist,
    Translations,
    Video,
    VideoFormat
};

/// Specifies what needs to be done in a processing job
#[derive(Clone, Debug)]
pub enum JobKind {
    AudioFormat {
        format: AudioFormat,
        video: Arc<Video>
    },
    AudioMeta {
        video: Arc<Video>
    },
    CollectionFastTasks(Arc<Collection>),
    PlaylistFastTasks(Arc<Playlist>),
    VideoFastTasks {
        poster_aspect: f32,
        video: Arc<Video>
    },
    VideoFormat {
        format: VideoFormat,
        video: Arc<Video>
    },
    VideoPickPoster {
        poster_aspect: f32,
        timecode: f32,
        video: Arc<Video>
    }
}

impl JobKind {
    pub fn is_high_priority(&self) -> bool {
        match self {
            JobKind::AudioMeta { .. } |
            JobKind::CollectionFastTasks(_) |
            JobKind::PlaylistFastTasks(_) |
            JobKind::VideoFastTasks { .. } |
            JobKind::VideoPickPoster { .. } => true,
            JobKind::AudioFormat { .. } |
            JobKind::VideoFormat { .. } => false
        }
    }

    pub fn representation(&self, translations: &Translations) -> String {
        match self {
            JobKind::AudioFormat { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Encode audio file for video '{label}'")
            }
            JobKind::AudioMeta { video } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute audio file metadata for video '{label}'")
            }
            JobKind::CollectionFastTasks(collection) => {
                let label = collection.title_or_slug_or_generic_label(translations);
                format!("Compute banner for collection '{label}'")
            }
            JobKind::PlaylistFastTasks(playlist) => {
                let label = playlist.title_or_slug_or_generic_label(translations);
                format!("Compute banner for playlist '{label}'")
            }
            JobKind::VideoFastTasks { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute metadata and/or poster for video '{label}'")
            }
            JobKind::VideoFormat { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Encode video file for video '{label}'")
            }
            JobKind::VideoPickPoster { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute random poster for video '{label}'")
            }
        }
    }
}

impl PartialEq for JobKind {
    fn eq(&self, other: &JobKind) -> bool {
        match self {
            JobKind::AudioFormat { format, video } => match other {
                JobKind::AudioFormat { format: other_format, video: other_video } =>
                    video.site_path == other_video.site_path && format == other_format,
                _ => false
            }
            JobKind::AudioMeta { video } => match other {
                JobKind::AudioMeta { video: other_video } =>
                    video.site_path == other_video.site_path,
                _ => false
            }
            JobKind::CollectionFastTasks(collection) => match other {
                JobKind::CollectionFastTasks(other_collection) => collection.site_path == other_collection.site_path,
                _ => false
            }
            JobKind::PlaylistFastTasks(playlist) => match other {
                JobKind::PlaylistFastTasks(other_playlist) => playlist.site_path == other_playlist.site_path,
                _ => false
            }
            JobKind::VideoFastTasks { poster_aspect, video } => match other {
                JobKind::VideoFastTasks { poster_aspect: other_poster_aspect, video: other_video } => {
                    video.site_path == other_video.site_path && poster_aspect == other_poster_aspect
                }
                _ => false
            }
            JobKind::VideoFormat { format, video } => match other {
                JobKind::VideoFormat { format: other_format, video: other_video } =>
                    video.site_path == other_video.site_path && format == other_format,
                _ => false
            }
            JobKind::VideoPickPoster { video, .. } => match other {
                JobKind::VideoPickPoster { video: other_video, .. } => video.site_path == other_video.site_path,
                _ => false
            }
        }
    }
}

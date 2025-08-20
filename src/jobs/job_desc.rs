// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use crate::{
    AudioFormat,
    Collection,
    Playlist,
    SitePath,
    Translations,
    Video,
    VideoFormat
};

/// The job description specifies what needs to be done in a processing job,
/// and to which resource (collection, playlist or video) it applies.
#[derive(Clone, Debug)]
pub enum JobDesc {
    AudioFormat {
        format: AudioFormat,
        video: Arc<Video>
    },
    AudioMeta {
        video: Arc<Video>
    },
    CollectionFastTasks {
        collection: Arc<Collection>
    },
    PlaylistFastTasks {
        playlist: Arc<Playlist>
    },
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

impl JobDesc {
    pub fn is_high_priority(&self) -> bool {
        match self {
            JobDesc::AudioMeta { .. } |
            JobDesc::CollectionFastTasks { .. } |
            JobDesc::PlaylistFastTasks { .. } |
            JobDesc::VideoFastTasks { .. } |
            JobDesc::VideoPickPoster { .. } => true,
            JobDesc::AudioFormat { .. } |
            JobDesc::VideoFormat { .. } => false
        }
    }

    pub fn label(&self, translations: &Translations) -> String {
        // TODO: Translate strings as soon as we reach some confidence on what they should contain
        match self {
            JobDesc::AudioFormat { format, video } => {
                let extension = format.extension();
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Encode audio file ({extension}) for video '{label}'")
            }
            JobDesc::AudioMeta { video } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute audio file metadata for video '{label}'")
            }
            JobDesc::CollectionFastTasks { collection } => {
                let label = collection.title_or_slug_or_generic_label(translations);
                format!("Compute banner for collection '{label}'")
            }
            JobDesc::PlaylistFastTasks { playlist } => {
                let label = playlist.title_or_slug_or_generic_label(translations);
                format!("Compute banner for playlist '{label}'")
            }
            JobDesc::VideoFastTasks { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute metadata and/or poster for video '{label}'")
            }
            JobDesc::VideoFormat { format, video } => {
                let format_label = format.label();
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Encode video file ({format_label}) for video '{label}'")
            }
            JobDesc::VideoPickPoster { video, .. } => {
                let label = video.title_or_slug_or_generic_label(translations);
                format!("Compute random poster for video '{label}'")
            }
        }
    }

    pub fn site_path(&self) -> &SitePath {
        match self {
            JobDesc::AudioFormat { video, .. } => &video.site_path,
            JobDesc::AudioMeta { video } => &video.site_path,
            JobDesc::CollectionFastTasks { collection } => &collection.site_path,
            JobDesc::PlaylistFastTasks { playlist } => &playlist.site_path,
            JobDesc::VideoFastTasks { video, .. } => &video.site_path,
            JobDesc::VideoFormat { video, .. } => &video.site_path,
            JobDesc::VideoPickPoster { video, .. } => &video.site_path,
        }
    }
}

impl PartialEq for JobDesc {
    fn eq(&self, other: &JobDesc) -> bool {
        match self {
            JobDesc::AudioFormat { format, video } => match other {
                JobDesc::AudioFormat { format: other_format, video: other_video } =>
                    video.site_path == other_video.site_path && format == other_format,
                _ => false
            }
            JobDesc::AudioMeta { video } => match other {
                JobDesc::AudioMeta { video: other_video } =>
                    video.site_path == other_video.site_path,
                _ => false
            }
            JobDesc::CollectionFastTasks { collection } => match other {
                JobDesc::CollectionFastTasks { collection: other_collection } =>
                    collection.site_path == other_collection.site_path,
                _ => false
            }
            JobDesc::PlaylistFastTasks { playlist } => match other {
                JobDesc::PlaylistFastTasks { playlist: other_playlist } =>
                    playlist.site_path == other_playlist.site_path,
                _ => false
            }
            JobDesc::VideoFastTasks { poster_aspect, video } => match other {
                JobDesc::VideoFastTasks { poster_aspect: other_poster_aspect, video: other_video } => {
                    video.site_path == other_video.site_path && poster_aspect == other_poster_aspect
                }
                _ => false
            }
            JobDesc::VideoFormat { format, video } => match other {
                JobDesc::VideoFormat { format: other_format, video: other_video } =>
                    video.site_path == other_video.site_path && format == other_format,
                _ => false
            }
            JobDesc::VideoPickPoster { video, .. } => match other {
                JobDesc::VideoPickPoster { video: other_video, .. } => video.site_path == other_video.site_path,
                _ => false
            }
        }
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use hyper8_translations::Translations;

use crate::{
    AudioFormat,
    Collection,
    Playlist,
    SitePath,
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

    /// A label for what this job produces.
    /// (e.g. "Video file (H264)" or "Audio metadata")
    pub fn output_label(&self, translations: &Translations) -> String {
        match self {
            JobDesc::AudioFormat { format, .. } => {
                translations.audio_file_xxx(format.extension())
            }
            JobDesc::AudioMeta { .. } => {
                translations.audio_metadata.to_string()
            }
            JobDesc::CollectionFastTasks { .. } => {
                translations.banner_image_variants.to_string()
            }
            JobDesc::PlaylistFastTasks { .. } => {
                translations.banner_image_variants.to_string()
            }
            JobDesc::VideoFastTasks { .. } => {
                translations.video_metadata_poster_image_and_variants.to_string()
            }
            JobDesc::VideoFormat { format, .. } => {
                translations.video_file_xxx(&format.label())
            }
            JobDesc::VideoPickPoster { .. } => {
                translations.poster_image.to_string()
            }
        }
    }

    /// The label of the collection, playlist or video for which is part of
    /// this job description.
    pub fn target_label(&self, translations: &Translations) -> &str {
        match self {
            JobDesc::CollectionFastTasks { collection } => {
                collection.title_or_slug_or_generic_label(translations)
            }
            JobDesc::PlaylistFastTasks { playlist } => {
                playlist.title_or_slug_or_generic_label(translations)
            }
            JobDesc::AudioFormat { video, .. } |
            JobDesc::AudioMeta { video } |
            JobDesc::VideoFastTasks { video, .. } |
            JobDesc::VideoFormat { video, .. } |
            JobDesc::VideoPickPoster { video, .. } => {
                video.title_or_slug_or_generic_label(translations)
            }
        }
    }

    /// Whether the JobDesc refers to media processing for the video with the
    /// given site_path
    pub fn targets_media(&self, site_path: &SitePath) -> bool {
        match self {
            JobDesc::AudioFormat { video, .. } |
            JobDesc::AudioMeta { video } |
            JobDesc::VideoFastTasks { video, .. } |
            JobDesc::VideoFormat { video, .. }  => {
                video.site_path == *site_path
            }
            JobDesc::CollectionFastTasks { .. } |
            JobDesc::PlaylistFastTasks { .. } |
            JobDesc::VideoPickPoster { .. } => false
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

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::{
    AudioMeta,
    Context,
    FileMeta,
    Video
};

/// Represents a physical audio file storing an audio-only version of a video.
/// For instance an OPUS and MP3 file containing audio for a video are both one
/// separate [AudioFile].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioFile {
    pub audio_meta: Option<AudioMeta>,
    pub download: Option<bool>,
    pub explicitly_defined: bool,
    pub file_meta: FileMeta,
    pub file_name: String,
    pub streaming: Option<bool>
}

impl AudioFile {
    pub const SUPPORTED_EXTENSIONS: [&str; 8] = [
        "aac",
        "aiff",
        "alac",
        "flac",
        "mp3",
        "ogg",
        "opus",
        "wav"
    ];

    pub fn audio_meta_unchecked(&self) -> &AudioMeta {
        self.audio_meta.as_ref().unwrap()
    }

    pub fn downloadable(&self) -> bool {
        self.download.is_none() ||
        self.download.is_some_and(|enabled| enabled)
    }

    pub fn mime_type(&self) -> &'static str {
        let (_stem, extension) = self.file_name.rsplit_once('.').unwrap();

        // This mapping must be kept up-to-date with
        // [SUPPORTED_EXTENSIONS], otherwise it's a full on-crash somewhere
        // (but at least something that is quickly and easily discovered).
        match extension {
            "aac" => "audio/aac",
            "aiff" => "audio/aiff",
            "alac" => "video/mp4; codecs=alac",
            "flac" => "audio/flac",
            "mp3" => "audio/mpeg",
            "ogg" => "audio/ogg",
            "opus" => "audio/ogg; codecs=opus",
            "wav" => "audio/wav",
            _ => unimplemented!()
        }
    }

    pub fn new(
        context: &Context,
        path: &Path,
        query_cached_audio_meta: bool,
        video: &Video
    ) -> AudioFile {
        let file_meta = FileMeta::new(&path);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let audio_meta = if query_cached_audio_meta {
            let version_path = format!("{}{}", &video.site_path.normalized(), file_name);

            context.cache.lock().unwrap().audio_meta
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
                .map(|cached| cached.audio_meta.clone())
        } else {
            None
        };

        AudioFile {
            audio_meta,
            download: None,
            explicitly_defined: false,
            file_meta,
            file_name,
            streaming: None
        }
    }

    pub fn new_from_manifest(
        context: &Context,
        download: Option<bool>,
        path: &Path,
        streaming: Option<bool>,
        video: &Video
    ) -> AudioFile {
        let file_meta = FileMeta::new(path);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let version_path = format!("{}{}", &video.site_path.normalized(), file_name);
        let audio_meta = context.cache.lock().unwrap().audio_meta
            .iter()
            .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
            .map(|cached| cached.audio_meta.clone());

        AudioFile {
            download,
            explicitly_defined: true,
            file_meta,
            file_name,
            streaming,
            audio_meta
        }
    }

    /// An audio file is considered "published" if streaming and/or download is
    /// enabled (and by default both are)
    pub fn published(&self) -> bool {
        self.downloadable() || self.streamable()
    }

    pub fn streamable(&self) -> bool {
        self.streaming.is_none() ||
        self.streaming.is_some_and(|enabled| enabled)
    }

    pub fn supported_extension(filename: &str) -> bool {
        Path::new(filename).extension().is_some_and(|extension| {
            AudioFile::SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported_extension| {
                    extension.eq_ignore_ascii_case(supported_extension)
                })
        })
    }
}

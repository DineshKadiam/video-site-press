// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_derive::{Deserialize, Serialize};

use crate::{AudioMeta, FileMeta};

/// Represents a physical audio file storing an audio-only version of a video.
/// For instance an OPUS and MP3 file containing audio for a video are both one
/// separate [AudioFile].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioFile {
    pub audio_meta: Option<AudioMeta>,
    pub file_meta: FileMeta,
    pub file_name: String
}

impl AudioFile {
    pub fn mime_type(&self) -> &'static str {
        let (_stem, extension) = self.file_name.rsplit_once('.').unwrap();

        // TODO: Mime types for less likely formats are to be added
        // This mapping must be kept up-to-date with what we accept in our
        // [supported_audio_extension] helper function, otherwise it's a full
        // on-crash (but at least something that is quickly and easily
        // discovered).
        match extension {
            "mp3" => "audio/mpeg",
            "opus" => "audio/ogg; codecs=opus",
            _ => unimplemented!()
        }
    }

    pub fn new(
        audio_meta: Option<AudioMeta>,
        file_meta: FileMeta,
        file_name: String
    ) -> AudioFile {
        AudioFile {
            audio_meta,
            file_meta,
            file_name
        }
    }

    pub fn audio_meta_unchecked(&self) -> &AudioMeta {
        self.audio_meta.as_ref().unwrap()
    }
}

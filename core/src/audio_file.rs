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
}

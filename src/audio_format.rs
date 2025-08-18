// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Opus
}

impl AudioFormat {
    pub fn extension(&self) -> &str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Opus => "opus"
        }
    }

    /// Determine the priority-based ordering between this and another format.
    /// Video is prioritized over audio, higher resolution over lower resolution.
    pub fn cmp_priority(&self, other: &AudioFormat) -> Ordering {
        match self {
            AudioFormat::Mp3 => match other {
                AudioFormat::Mp3 => Ordering::Equal,
                AudioFormat::Opus => Ordering::Less
            }
            AudioFormat::Opus => match other {
                AudioFormat::Mp3 => Ordering::Greater,
                AudioFormat::Opus => Ordering::Equal
            }
        }
    }
}

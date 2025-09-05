// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_derive::Serialize;

use super::FFmpegEncodingStats;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    /// Progress as a factor in the range 0-1
    pub factor: f32,
    /// How long the job already took (in seconds)
    pub duration: f32,
    /// How long the job is estimated to still take (in seconds)
    pub duration_remaining: f32,
    /// Progress in percent
    pub percent: u8
}

impl JobProgress {
    pub fn new(
        duration: f32,
        stats: FFmpegEncodingStats,
        video_duration: f32
    ) -> JobProgress {
        let factor = (stats.time / video_duration).clamp(0.0, 1.0);

        let duration_remaining = (1.0 - factor) * (duration / factor);
        let percent = (factor * 100.0) as u8;

        JobProgress {
            factor,
            duration,
            duration_remaining,
            percent
        }
    }
}

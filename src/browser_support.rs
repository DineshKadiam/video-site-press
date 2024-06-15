// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Language, VideoMeta};

/// Compatible (Ok) and Incompatible (Err) include a message that describes
/// why the format was deemed (in)compatible.
pub fn analysis(
    file_name: &str,
    language: &Language,
    video_meta: &VideoMeta
) -> Option<Result<String, String>> {
    let mut supported: Vec<String> = Vec::new();
    let unsupported: Vec<String> = Vec::new();

    let t_generally_contains_codecs_supported_in_browsers = language.translations.generally_contains_codecs_supported_in_browsers;
    let t_generally_supported_in_browsers = language.translations.generally_supported_in_browsers;

    // TODO: .mkv should be reported as an unknown - possibly containing unsupported things because it can contain anything
    if file_name.ends_with(".webm") {
        let message = format!("WebM container: {t_generally_contains_codecs_supported_in_browsers}");
        supported.push(message);
    }

    if video_meta.audio_codec.as_ref().is_some_and(|codec| codec == "aac") {
        let message = format!("AAC audio codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    }

    if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "h264") {
        let message = format!("H264 video codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    } else if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "vp9") {
        let message = format!("VP9 video codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    }

    if !unsupported.is_empty() {
        Some(Err(unsupported.join(", ")))
    } else if !supported.is_empty() {
        Some(Ok(supported.join(", ")))
    } else {
        None
    }
}
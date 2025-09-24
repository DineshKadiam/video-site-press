// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_translations::Translations;

use crate::{AudioMeta, VideoMeta};

pub enum BrowserSupport {
    Supported(BrowserSupportVerdict),
    Unknown,
    Unsupported(BrowserSupportVerdict)
}

pub struct BrowserSupportVerdict {
    pub explanation: String,
    pub summary: String
}

impl BrowserSupportVerdict {
    pub fn new(explanation: String, summary: String) -> BrowserSupportVerdict {
        BrowserSupportVerdict {
            explanation,
            summary
        }
    }
}

impl BrowserSupport {
    pub fn audio_analysis(
        audio_meta: &AudioMeta,
        file_name: &str,
        translations: &Translations
    ) -> BrowserSupport {
        let mut summary: Vec<String> = Vec::new();
        let mut supported: Vec<String> = Vec::new();
        let unsupported: Vec<String> = Vec::new();

        let t_audio_codec = translations.audio_codec;
        let t_container = translations.container;
        let t_generally_supported_in_browsers = translations.generally_supported_in_browsers;

        if file_name.ends_with(".aac") {
            let label = format!("AAC ({t_container})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if file_name.ends_with(".mp3") {
            let label = format!("MP3 ({t_container})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if file_name.ends_with(".opus") {
            let label = format!("Opus ({t_container})");
            // TODO: Add note regarding older Safari (and misc exotic) browsers that do not support it
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if file_name.ends_with(".wav") {
            let label = format!("WAV ({t_container})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if audio_meta.codec.as_ref().is_some_and(|codec| codec == "aac") {
            let label = format!("AAC ({t_audio_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        } else if audio_meta.codec.as_ref().is_some_and(|codec| codec == "mp3") {
            let label = format!("MP3 ({t_audio_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        } else if audio_meta.codec.as_ref().is_some_and(|codec| codec == "opus") {
            let label = format!("Opus ({t_audio_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if !unsupported.is_empty() {
            let verdict = BrowserSupportVerdict::new(
                unsupported.join("\n"),
                summary.join(", ")
            );
            BrowserSupport::Unsupported(verdict)
        } else if !supported.is_empty() {
            let verdict = BrowserSupportVerdict::new(
                supported.join("\n"),
                summary.join(", ")
            );
            BrowserSupport::Supported(verdict)
        } else {
            BrowserSupport::Unknown
        }
    }

    pub fn video_analysis(
        file_name: &str,
        translations: &Translations,
        video_meta: &VideoMeta
    ) -> BrowserSupport {
        let mut summary: Vec<String> = Vec::new();
        let mut supported: Vec<String> = Vec::new();
        let unsupported: Vec<String> = Vec::new();

        let t_audio_codec = translations.audio_codec;
        let t_container = translations.container;
        let t_generally_contains_codecs_supported_in_browsers = translations.generally_contains_codecs_supported_in_browsers;
        let t_generally_supported_in_browsers = translations.generally_supported_in_browsers;
        let t_video_codec = translations.video_codec;

        // TODO: .mkv should be reported as an unknown - possibly containing
        // unsupported things because it can contain anything

        if file_name.ends_with(".webm") {
            let label = format!("WebM ({t_container})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_contains_codecs_supported_in_browsers}");
            supported.push(message);
        }

        if video_meta.audio_codec.as_ref().is_some_and(|codec| codec == "aac") {
            let label = format!("AAC ({t_audio_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "h264") {
            let label = format!("H264 ({t_video_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        } else if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "vp8") {
            let label = format!("VP8 ({t_video_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        } else if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "vp9") {
            let label = format!("VP9 ({t_video_codec})");
            summary.push(label.clone());
            let message = format!("{label}: {t_generally_supported_in_browsers}");
            supported.push(message);
        }

        if !unsupported.is_empty() {
            let verdict = BrowserSupportVerdict::new(
                unsupported.join("\n"),
                summary.join(", ")
            );
            BrowserSupport::Unsupported(verdict)
        } else if !supported.is_empty() {
            let verdict = BrowserSupportVerdict::new(
                supported.join("\n"),
                summary.join(", ")
            );
            BrowserSupport::Supported(verdict)
        } else {
            BrowserSupport::Unknown
        }
    }
}

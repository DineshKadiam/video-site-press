// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

use hyper8_translations::Translations;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum MediaUsage {
    Download,
    Offline,
    Streaming,
    Unrestricted
}

impl MediaUsage {
    pub fn from_key_unchecked(key: &str) -> MediaUsage {
        match key {
            "download" => MediaUsage::Download,
            "offline" => MediaUsage::Offline,
            "streaming" => MediaUsage::Streaming,
            "unrestricted" => MediaUsage::Unrestricted,
            _ => unreachable!()
        }
    }

    pub fn key(&self) -> &str {
        match self {
            MediaUsage::Download => "download",
            MediaUsage::Offline => "offline",
            MediaUsage::Streaming => "streaming",
            MediaUsage::Unrestricted => "unrestricted"
        }
    }

    pub fn variants(translations: &Translations) -> [(&str, MediaUsage); 4] {
        [
            (translations.no_usage_restrictions, MediaUsage::Unrestricted),
            (translations.streaming_only, MediaUsage::Streaming),
            (translations.download_only, MediaUsage::Download),
            (translations.keep_offline, MediaUsage::Offline)
        ]
    }
}

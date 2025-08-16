// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoOrder {
    Default,
    ReleaseDateAsc,
    ReleaseDateDesc,
    SortNumberAsc,
    SortNumberDesc,
    TitleAsc,
    TitleDesc
}

impl VideoOrder {
    const SUPPORTED_KEYS: [&str; 7] = [
        "default",
        "release_date_asc",
        "release_date_desc",
        "sort_number_asc",
        "sort_number_desc",
        "title_asc",
        "title_desc"
    ];

    pub fn from_key(key: &str) -> Option<VideoOrder> {
        match key {
            "default" => Some(VideoOrder::Default),
            "release_date_asc" => Some(VideoOrder::ReleaseDateAsc),
            "release_date_desc" => Some(VideoOrder::ReleaseDateDesc),
            "sort_number_asc" => Some(VideoOrder::SortNumberAsc),
            "sort_number_desc" => Some(VideoOrder::SortNumberDesc),
            "title_asc" => Some(VideoOrder::TitleAsc),
            "title_desc" => Some(VideoOrder::TitleDesc),
            _ => None
        }
    }

    pub fn key(&self) -> &str {
        match self {
            VideoOrder::Default => "default",
            VideoOrder::ReleaseDateAsc => "release_date_asc",
            VideoOrder::ReleaseDateDesc => "release_date_desc",
            VideoOrder::SortNumberAsc => "sort_number_asc",
            VideoOrder::SortNumberDesc => "sort_number_desc",
            VideoOrder::TitleAsc => "title_asc",
            VideoOrder::TitleDesc => "title_desc"
        }
    }

    pub fn supported_keys() -> String {
        VideoOrder::SUPPORTED_KEYS
            .iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

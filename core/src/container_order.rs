// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContainerOrder {
    Default,
    SortNumberAsc,
    SortNumberDesc,
    TitleAsc,
    TitleDesc
}

impl ContainerOrder {
    const SUPPORTED_KEYS: [&str; 5] = [
        "default",
        "sort_number_asc",
        "sort_number_desc",
        "title_asc",
        "title_desc"
    ];

    pub fn from_key(key: &str) -> Option<ContainerOrder> {
        match key {
            "default" => Some(ContainerOrder::Default),
            "sort_number_asc" => Some(ContainerOrder::SortNumberAsc),
            "sort_number_desc" => Some(ContainerOrder::SortNumberDesc),
            "title_asc" => Some(ContainerOrder::TitleAsc),
            "title_desc" => Some(ContainerOrder::TitleDesc),
            _ => None
        }
    }

    pub fn key(&self) -> &str {
        match self {
            ContainerOrder::Default => "default",
            ContainerOrder::SortNumberAsc => "sort_number_asc",
            ContainerOrder::SortNumberDesc => "sort_number_desc",
            ContainerOrder::TitleAsc => "title_asc",
            ContainerOrder::TitleDesc => "title_desc"
        }
    }

    pub fn supported_keys() -> String {
        ContainerOrder::SUPPORTED_KEYS
            .iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

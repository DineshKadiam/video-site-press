// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use hyper8_core::{
    Asc,
    Collection,
    Comparison,
    Container,
    Desc,
    Language,
    Video,
    VideoOrder
};
use hyper8_translations::Translations;

#[derive(Clone, Copy, Debug)]
pub enum TreeViewOrder {
    Default,
    SiteOrder,
    TitleAsc,
    TitleDesc
}

impl TreeViewOrder {
    pub fn cycle(&self) -> TreeViewOrder {
        match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => TreeViewOrder::TitleAsc,
            TreeViewOrder::TitleAsc => TreeViewOrder::TitleDesc,
            TreeViewOrder::TitleDesc => TreeViewOrder::SiteOrder,
        }
    }

    pub fn from_key(key: &str) -> Option<TreeViewOrder> {
        match key {
            "site_order" => Some(TreeViewOrder::SiteOrder),
            "title_asc" => Some(TreeViewOrder::TitleAsc),
            "title_desc" => Some(TreeViewOrder::TitleDesc),
            _ => None
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => "site_order",
            TreeViewOrder::TitleAsc => "title_asc",
            TreeViewOrder::TitleDesc => "title_desc"
        }
    }

    pub fn label(&self, translations: &Translations) -> String {
        match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => translations.site_order.to_string(),
            TreeViewOrder::TitleAsc => format!("{} A-Z", translations.title),
            TreeViewOrder::TitleDesc => format!("{} Z-A", translations.title)
        }
    }

    pub fn ordered_containers(
        &self,
        collection: &Collection,
        language: &Language
    ) -> Vec<Container> {
        let mut containers = Vec::new();

        for playlist in &collection.playlists {
            containers.push(Container::Playlist(playlist.clone()));
        }

        for subcollection in &collection.subcollections {
            containers.push(Container::Collection(subcollection.clone()));
        }

        let (comparison, direction) = match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder =>
                Container::container_order_comparison_and_direction(&collection.container_order),
            TreeViewOrder::TitleAsc =>
                (Container::cmp_slug_then_title as Comparison<Container>, Asc),
            TreeViewOrder::TitleDesc =>
                (Container::cmp_slug_then_title as Comparison<Container>, Desc)
        };

        containers.sort_unstable_by(|a, b| comparison(a, b, &language.icu_collator, direction));

        containers
    }

    pub fn ordered_videos(
        &self,
        language: &Language,
        video_order: &VideoOrder,
        videos: &[Arc<Video>]
    ) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = videos.to_vec();

        let (comparison, direction) = match self {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => Video::video_order_comparison_and_direction(video_order),
            TreeViewOrder::TitleAsc => (Video::cmp_slug_then_title as Comparison<Video>, Asc),
            TreeViewOrder::TitleDesc => (Video::cmp_slug_then_title as Comparison<Video>, Desc)
        };

        videos.sort_unstable_by(|a, b| comparison(a, b, &language.icu_collator, direction));

        videos
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    Collection,
    ContainerOrder,
    Playlist
};

/// This abstraction allows us to mix collections and playlists together in a
/// single Vec<Container> in order to sort them for display and rendering in
/// the editor and built site.
pub enum Container {
	Collection(Arc<Collection>),
	Playlist(Arc<Playlist>)
}

impl Container {
    // Returns a comparator function according to the passed [ContainerOrder]
    pub fn cmp_by_container_order(
        video_order: &ContainerOrder
    ) -> fn(&Container, &Container) -> Ordering {
        match video_order {
            ContainerOrder::Default => Container::cmp_by_sort_number_asc_then_latest_release_date_desc,
            ContainerOrder::SortNumberAsc => Container::cmp_by_sort_number_asc,
            ContainerOrder::SortNumberDesc => Container::cmp_by_sort_number_desc,
            ContainerOrder::TitleAsc => Container::cmp_by_title_or_slug_asc,
            ContainerOrder::TitleDesc => Container::cmp_by_title_or_slug_desc
        }
    }

    pub fn cmp_by_latest_release_date_desc(a: &Container, b: &Container) -> Ordering {
        b.latest_release_date().cmp(&a.latest_release_date())
    }

    // Containers without title are sorted to the front here, so the group of
    // containers with title is sorted in itself but always kept separate
    // from the group of containers with only a slug, which is also sorted in
    // itself.
    pub fn cmp_by_slug_then_title_asc(a: &Container, b: &Container) -> Ordering {
        match (&a.title(), &b.title()) {
            (Some(a_title), Some(b_title)) => a_title.cmp(b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    }

    // Containers without title are sorted to the front here, so the group of
    // containers with title is sorted in itself but always kept separate
    // from the group of containers with only a slug, which is also sorted in
    // itself.
    pub fn cmp_by_slug_then_title_desc(a: &Container, b: &Container) -> Ordering {
        match (&a.title(), &b.title()) {
            (Some(a_title), Some(b_title)) => b_title.cmp(a_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => b.slug().cmp(&a.slug())
        }
    }

    // Containers without sort number are sorted to the back
    pub fn cmp_by_sort_number_asc(a: &Container, b: &Container) -> Ordering {
        match (&a.sort_number(), &b.sort_number()) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Containers without sort number are sorted to the back
    pub fn cmp_by_sort_number_asc_then_latest_release_date_desc(
        a: &Container,
        b: &Container
    ) -> Ordering {
        match (&a.sort_number(), &b.sort_number()) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Container::cmp_by_latest_release_date_desc(a, b)
        }
    }

    // Containers without sort number are sorted to the back
    pub fn cmp_by_sort_number_desc(a: &Container, b: &Container) -> Ordering {
        match (&a.sort_number(), &b.sort_number()) {
            (Some(a_sort_number), Some(b_sort_number)) => b_sort_number.cmp(a_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Containers with title and containers with only a slug are intermixed
    // and sorted together here.
    pub fn cmp_by_title_or_slug_asc(a: &Container, b: &Container) -> Ordering {
        let a_title = a.title();
        let b_title = b.title();

        let a_title_or_slug = a_title.as_deref().or_else(|| a.slug());
        let b_title_or_slug = b_title.as_deref().or_else(|| b.slug());

        a_title_or_slug.cmp(&b_title_or_slug)
    }

    // Containers with title and containers with only a slug are intermixed
    // and sorted together here.
    pub fn cmp_by_title_or_slug_desc(a: &Container, b: &Container) -> Ordering {
        let a_title = a.title();
        let b_title = b.title();

        let a_title_or_slug = a_title.as_deref().or_else(|| a.slug());
        let b_title_or_slug = b_title.as_deref().or_else(|| b.slug());

        b_title_or_slug.cmp(&a_title_or_slug)
    }

    /// Only used internally so we can have a cleaner implementation of the
    /// sort compare methods.
    fn latest_release_date(&self) -> Option<NaiveDate> {
        match self {
            Container::Collection(collection) => collection.latest_release_date(),
            Container::Playlist(playlist) => playlist.latest_release_date()
        }
    }

    /// Only used internally so we can have a cleaner implementation of the
    /// sort compare methods.
    fn slug(&self) -> Option<&str> {
        match self {
            Container::Collection(collection) => collection.slug(),
            Container::Playlist(playlist) => playlist.slug()
        }
    }

    /// Only used internally so we can have a cleaner implementation of the
    /// sort compare methods.
    fn sort_number(&self) -> Option<i32> {
        match self {
            Container::Collection(collection) => collection.sort_number,
            Container::Playlist(playlist) => playlist.sort_number
        }
    }

    /// Only used internally so we can have a cleaner implementation of the
    /// sort compare methods.
    fn title(&self) -> Option<&str> {
        match self {
            Container::Collection(collection) => collection.title.as_deref(),
            Container::Playlist(playlist) => playlist.title.as_deref()
        }
    }
}

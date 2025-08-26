// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::sync::Arc;

use chrono::NaiveDate;
use icu_collator::CollatorBorrowed;

use crate::{
    Asc,
    Collection,
    Comparison,
    ContainerOrder,
    Desc,
    Playlist,
    SortDirection
};

/// This abstraction allows us to mix collections and playlists together in a
/// single Vec<Container> in order to sort them for display and rendering in
/// the editor and built site.
pub enum Container {
	Collection(Arc<Collection>),
	Playlist(Arc<Playlist>)
}

impl Container {
    /// Containers without title are always sorted to the front here -
    /// regardless of sort direction - so the group of containers with title
    /// is sorted in itself but always kept separate from the group of
    /// containers with only a slug, which is also sorted in itself.
    pub fn cmp_slug_then_title(
        a: &Container,
        b: &Container,
        collator: &CollatorBorrowed,
        direction: SortDirection
    ) -> Ordering {
        match (&a.title(), &b.title()) {
            (Some(a_title), Some(b_title)) => {
                let ordering = collator.compare(a_title, b_title);
                direction.orient(ordering)
            }
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => {
                let ordering = collator.compare(a.slug_unchecked(), b.slug_unchecked());
                direction.orient(ordering)
            }
        }
    }

    /// Containers without sort number are sorted to the back, regardless of
    /// sort direction
    pub fn cmp_sort_number(
        a: &Container,
        b: &Container,
        _: &CollatorBorrowed,
        direction: SortDirection
    ) -> Ordering {
        match (&a.sort_number(), &b.sort_number()) {
            (Some(a_sort_number), Some(b_sort_number)) => {
                let ordering = a_sort_number.cmp(b_sort_number);
                direction.orient(ordering)
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    /// For instance for ascending sorting, this sorts by sort number
    /// ascending, but all containers without sort number are then sorted
    /// inversely (in this case descending) by the latest release date of any
    /// video they contain at any hierarchical depth. Containers without sort
    /// number are sorted to the back, regardless of sort direction.
    pub fn cmp_sort_number_then_latest_release_date_inverse(
        a: &Container,
        b: &Container,
        collator: &CollatorBorrowed,
        direction: SortDirection
    ) -> Ordering {
        match (&a.sort_number(), &b.sort_number()) {
            (Some(a_sort_number), Some(b_sort_number)) => {
                let ordering = a_sort_number.cmp(b_sort_number);
                direction.orient(ordering)
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => {
                Container::cmp_latest_release_date(a, b, collator, direction.inverse())
            }
        }
    }

    /// Containers with title and containers with only a slug are intermixed
    /// and sorted together here.
    pub fn cmp_title_or_slug(
        a: &Container,
        b: &Container,
        collator: &CollatorBorrowed,
        direction: SortDirection
    ) -> Ordering {
        let a_title_or_slug = a.title().unwrap_or_else(|| a.slug_unchecked());
        let b_title_or_slug = b.title().unwrap_or_else(|| b.slug_unchecked());

        let ordering = collator.compare(a_title_or_slug, b_title_or_slug);

        direction.orient(ordering)
    }

    /// Returns a comparison function and sort direction according to the
    /// passed [ContainerOrder]
    pub fn container_order_comparison_and_direction(
        video_order: &ContainerOrder
    ) -> (Comparison<Container>, SortDirection) {
        match video_order {
            ContainerOrder::Default =>
                (Container::cmp_sort_number_then_latest_release_date_inverse, Asc),
            ContainerOrder::SortNumberAsc => (Container::cmp_sort_number, Asc),
            ContainerOrder::SortNumberDesc => (Container::cmp_sort_number, Desc),
            ContainerOrder::TitleAsc => (Container::cmp_title_or_slug, Asc),
            ContainerOrder::TitleDesc => (Container::cmp_title_or_slug, Desc)
        }
    }

    pub fn cmp_latest_release_date(
        a: &Container,
        b: &Container,
        _: &CollatorBorrowed,
        direction: SortDirection
    ) -> Ordering {
        let ordering = a.latest_release_date().cmp(&b.latest_release_date());
        direction.orient(ordering)
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
    fn slug_unchecked(&self) -> &str {
        match self {
            Container::Collection(collection) => collection.slug_unchecked(),
            Container::Playlist(playlist) => playlist.slug_unchecked()
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

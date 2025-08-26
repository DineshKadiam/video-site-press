// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

use icu_collator::CollatorBorrowed;

/// Sorting for containers (collections and playlists) and videos always uses
/// standardized comparison functions that take in the two items to be
/// compared as well as a collator and sort direction. Note that the collator
/// is sometimes not actually used in the comparison, but currently included
/// in order to avoid complex abstractions to satisfy Rust's type system.
pub type Comparison<T> = fn(&T, &T, &CollatorBorrowed, SortDirection) -> Ordering;

/// We always implement cmp_* functions in ascending order, but finally apply
/// a sort direction to conditionally reverse the direction where applies.
#[derive(Clone, Copy)]
pub enum SortDirection {
    Asc,
    Desc
}

pub use SortDirection::{Asc, Desc};

impl SortDirection {
    pub fn inverse(&self) -> SortDirection {
        match self {
            Asc => Desc,
            Desc => Asc
        }
    }

    /// Any comparison in rust defaults to ascending order, hence this this is
    /// also the way in which all comparisons in Hyper 8 are implemented.
    /// Consequently, we can take the result of any of our comparisons and
    /// either leave it untouched (Asc) or reverse it (Desc). This is what
    /// orient() is for, used at the end of each of our cmp_* functions.
    pub fn orient(&self, ordering: Ordering) -> Ordering {
        match self {
            Asc => ordering,
            Desc => ordering.reverse()
        }
    }
}

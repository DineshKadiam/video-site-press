// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{MAIN_SEPARATOR_STR, Path};

use serde_derive::Deserialize;

use crate::Context;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SitePath {
	/// Basically "", "foo/", "foo/bar/", etc.
	/// Renaming for serde is done so we can directly use this struct
	/// to deserialize Actix path segments in urls to a SitePath struct.
	#[serde(rename(deserialize = "site_path"))]
	normalized: String
}

impl SitePath {
	/// Returns the closest descendant to go from this site path to
	/// another site path:
	/// - "foo/" is the closest descendant to go from "" to "foo/bar/" (or "foo/bar/...")
	/// - "foo/bar/" is the closests descendant to go from "foo/" to "foo/bar/baz" (or "foo/bar/baz/...")
	/// - etc.
	pub fn closest_descendant_to_unchecked(&self, other: &SitePath) -> SitePath {
		let remainder = other.normalized.strip_prefix(&self.normalized).unwrap();
		let closest_descendant = remainder.split_inclusive('/').nth(1).unwrap();
		SitePath::new(self.normalized.clone() + closest_descendant)
	}

	/// This returns the internally stored, normalized representation, so
	/// something like "", "foo/", "foo/bar/", etc. on platforms where the
	/// separator is the forward slash, otherwise it converts to a backward
	/// slash separated representation, so something
	/// like "", "foo\", "foo\bar\", etc.
	pub fn filesystem_relative(&self) -> String {
		if MAIN_SEPARATOR_STR == "/" {
			self.normalized.clone()
		} else {
			self.normalized.replace('/', MAIN_SEPARATOR_STR)
		}
	}

	pub fn from_path(context: &Context, path: &Path) -> SitePath {
        let normalized = path
            .strip_prefix(&context.site_dir)
            .unwrap()
            .iter()
            .map(|os_str| format!("{}/", os_str.to_str().unwrap()))
            .collect::<Vec<String>>()
            .join("");

        SitePath::new(normalized)
	}

	pub fn last_component(&self) -> Option<&str> {
		match self.normalized.is_empty() {
			true => None,
			false => self.normalized.rsplit_terminator('/').next()
		}
	}

	/// Whether this site path is a direct descendant (child) of
	/// the other site path, or in other terms, whether the other
	/// site path is the direct parent of this site path:
	/// - "foo/" is direct descendant of ""
	/// - "foo/bar/" is direct descendant of "foo/"
	/// - etc.
	pub fn is_direct_descendant_of(&self, other: &SitePath) -> bool {
		match self.normalized.strip_prefix(&other.normalized) {
			Some(remainder) => remainder.matches('/').count() == 1,
			None => false
		}
	}

	pub fn is_root_path(&self) -> bool {
		self.normalized.is_empty()
	}

	pub fn join(&self, slug: &str) -> SitePath {
		let joined = format!("{}{}/", self.normalized, slug);
		SitePath::new(joined)
	}

	pub fn last_component_unchecked(&self) -> &str {
		self.normalized.rsplit_terminator('/').next().unwrap()
	}

	fn new(normalized: String) -> SitePath {
		SitePath {
			normalized
		}
	}

	/// This returns the internally stored, normalized representation, so
	/// something like "", "foo/", "foo/bar/", etc.
	pub fn normalized(&self) -> &str {
		&self.normalized
	}

	// Calling this on a root node returns the same site path again
	pub fn parent_site_path_unchecked(&self) -> SitePath {
		if self.normalized.is_empty() {
			SitePath::root_path()
		} else {
			let last_slash_trimmed = &self.normalized[..self.normalized.len() - 1];

			match last_slash_trimmed.rfind('/') {
				Some(next_last_slash_index) => SitePath::new(String::from(&last_slash_trimmed[..next_last_slash_index + 1])),
				None => SitePath::root_path()
			}
		}
	}

	pub fn root_path() -> SitePath {
		SitePath {
			normalized: String::new()
		}
	}

	pub fn starts_with(&self, other: &SitePath) -> bool {
		self.normalized.starts_with(&other.normalized)
	}
}

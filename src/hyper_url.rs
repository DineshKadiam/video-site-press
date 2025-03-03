// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use crate::Site;

/// Urls can be user-provided with or without trailing slash
/// ("https://example.com/subdir" "https://example.com/subdir/"), but for
/// further url construction we want to ensure a normalized format, therefore
/// we store both the representation (with or without trailing slash, as
/// provided by the user, this is what they see) and the normalized form
/// (this is what we internally use to construct urls).
#[derive(Clone, Debug)]
pub struct HyperUrl {
    /// Always with a trailing slash, this is the internally used,
    /// normalized form.
    pub normalized: String,
    /// With or without trailing slash, as provided by the user.
    pub representation: String
}

impl HyperUrl {
    pub fn is_https(&self) -> bool {
        self.normalized.starts_with("https")
    }

    pub fn join(&self, path: &str, site: &Site) -> String {
        let normalized = &self.normalized;

        if path.is_empty() {
            match site.clean_urls {
                true => normalized.clone(),
                false => format!("{normalized}index.html")
            }
        } else {
            match site.clean_urls {
                true => format!("{normalized}{path}/"),
                false => format!("{normalized}{path}/index.html")
            }
        }
    }

    pub fn join_prefix(&self, path: &str) -> String {
        let normalized = &self.normalized;

        match path.is_empty() {
            true => normalized.clone(),
            false => format!("{normalized}{path}/")
        }
    }

    /// Delegates parsing to the url crate and stores the url as both a
    /// normalized form (with a guaranteed trailing slash) and representation
    /// variant (with or without trailing slash, as provided by the user).
    pub fn parse(url: impl Into<String>) -> Result<HyperUrl, String> {
        let representation = url.into();
        let normalized = match representation.ends_with('/') {
            true => representation.clone(),
            false => format!("{representation}/")
        };

        match Url::parse(&normalized) {
            Ok(_) => Ok(HyperUrl { normalized, representation }),
            Err(err) => Err(err.to_string())
        }
    }
}

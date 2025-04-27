// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use crate::{Site, SitePath};

/// Urls can be user-provided with or without trailing slash
/// ("https://example.com/subdir" "https://example.com/subdir/"), but for
/// further url construction we want to ensure a normalized format, therefore
/// we store both the representation (with or without trailing slash, as
/// provided by the user, this is what they see) and the normalized form
/// (this is what we internally use to construct urls).
#[derive(Clone, Debug)]
pub struct SiteUrl {
    /// Always with a trailing slash, this is the internally used,
    /// normalized form.
    pub normalized: String,
    /// With or without trailing slash, as provided by the user.
    pub representation: String
}

impl SiteUrl {
    pub fn example_url() -> SiteUrl {
        SiteUrl {
            normalized: String::from("https://example.com/"),
            representation: String::from("https://example.com/")
        }
    }

    pub fn is_https(&self) -> bool {
        self.normalized.starts_with("https")
    }

    /// Joins the passed site path plus a (sub-)dir with the intention of
    /// constructing an index page url, i.e. one that points to the dir or
    /// the index.html file inside it (depending on the clean_urls option).
    pub fn join_dir(&self, site: &Site, dir: &str, site_path: &SitePath) -> String {
        let mut url = self.normalized.clone();

        if !site_path.is_root_path() {
            url.push_str(site_path.normalized());
        }

        url.push_str(dir);

        if !site.clean_urls {
            url.push_str("index.html");
        }

        url
    }

    /// Joins the passed site path plus a file with the intention of
    /// constructing a direct file url.
    pub fn join_file(&self, file: &str, site_path: &SitePath) -> String {
        let mut url = self.normalized.clone();

        if !site_path.is_root_path() {
            url.push_str(site_path.normalized());
        }

        url.push_str(file);

        url
    }


    /// Joins the passed site path with the intention of constructing an index
    /// page url, i.e. one that points to a directory or the index.html file
    /// inside it (depending on the clean_urls option).
    pub fn join_index(&self, site: &Site, site_path: &SitePath) -> String {
        let mut url = self.normalized.clone();

        if !site_path.is_root_path() {
            url.push_str(site_path.normalized());
        }

        if !site.clean_urls {
            url.push_str("index.html");
        }

        url
    }

    /// Joins the passed site path with the intention of constructing a url
    /// that is to be extended afterwards, i.e. one that points to a
    /// directory only (so for instance a filename can be appended).
    pub fn join_prefix(&self, site_path: &SitePath) -> String {
        let mut url = self.normalized.clone();

        if !site_path.is_root_path() {
            url.push_str(site_path.normalized());
        }

        url
    }

    /// Delegates parsing to the url crate and stores the url as both a
    /// normalized form (with a guaranteed trailing slash) and representation
    /// variant (with or without trailing slash, as provided by the user).
    pub fn parse(url: impl Into<String>) -> Result<SiteUrl, String> {
        let representation = url.into();
        let normalized = match representation.ends_with('/') {
            true => representation.clone(),
            false => format!("{representation}/")
        };

        match Url::parse(&normalized) {
            Ok(_) => Ok(SiteUrl { normalized, representation }),
            Err(err) => Err(err.to_string())
        }
    }

    /// Returns this url without http(s):// and without trailing slash.
    pub fn without_scheme_and_trailing_slash(&self) -> &str {
        let without_scheme = self.normalized.split_once("://").unwrap().1;
        &without_scheme[..(without_scheme.len() - 1)]
    }
}

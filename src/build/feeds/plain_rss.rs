// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! RSS 2.0 specification for reference:
//! https://www.rssboard.org/rss-specification

use std::fs;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    Context,
    Site,
    SitePath,
    Video
};

use super::PLAIN_RSS_FILENAME;
use super::rss::rss;

pub fn plain_rss(
    build_time: DateTime<Utc>,
    context: &Context,
    description: &Option<String>,
    site: &Site,
    site_path: &SitePath,
    title: &str,
    videos: &[Arc<Video>]
) {
    let feed_url = site.absolute_url_file_unchecked(PLAIN_RSS_FILENAME, site_path);

    // The plain RSS feed just re-uses the plain RSS base markup in the
    // rss module, adding nothing at all.
    let channel_extensions = "";

    let extra_namespaces = &[];

    let xml = rss(
        build_time,
        channel_extensions,
        description,
        extra_namespaces,
        &feed_url,
        &mut item_extensions,
        site,
        title,
        videos
    );

    let path = context.build_dir.join(site_path.filesystem_relative()).join(PLAIN_RSS_FILENAME);
    fs::write(path, xml).unwrap();
}

pub fn item_extensions(
    _build_time: DateTime<Utc>,
    _site: &Site,
    _video: &Video
) -> String {
    // The plain RSS feed just re-uses the plain RSS base markup in the
    // rss module, adding nothing at all.
    String::new()
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use chrono::Utc;
use hyper8_translations::Translations;
use indoc::formatdoc;

use crate::{
    Context,
    Site,
    SitePath,
    Video
};

mod atom;
mod media_rss;
mod plain_rss;
mod podcast_rss;
mod rss;

pub const ATOM_FILENAME: &str = "feed.atom";
pub const PLAIN_RSS_FILENAME: &str = "plain.rss";
pub const MEDIA_RSS_FILENAME: &str = "media.rss";
pub const PODCAST_RSS_FILENAME: &str = "podcast.rss";

/// Generate all feeds, writing them to the build directory.
pub fn generate(
    context: &Context,
    description: &Option<String>,
    site: &Site,
    site_path: &SitePath,
    title: &str,
    videos: &Vec<Arc<Video>>
) {
    let build_time = Utc::now();

    atom::atom(
        build_time,
        context,
        description,
        site,
        site_path,
        title,
        videos
    );

    media_rss::media_rss(
        build_time,
        context,
        description,
        site,
        site_path,
        title,
        videos
    );

    plain_rss::plain_rss(
        build_time,
        context,
        description,
        site,
        site_path,
        title,
        videos
    );

    podcast_rss::podcast_rss(
        build_time,
        context,
        description,
        site,
        site_path,
        title,
        videos
    );
}

/// <link> tags to be placed in the <head> of the page to point browser to
/// available feeds for the page. This assumes that all feed files are placed
/// exactly in the same directory, i.e. from the page /foo/bar/ it is assumed
/// that feed files are /foo/bar/feed.atom, etc.
pub fn meta_link_tags(translations: &Translations) -> String {
    let atom_filename = ATOM_FILENAME;
    let media_rss_filename = MEDIA_RSS_FILENAME;
    let plain_rss_filename = PLAIN_RSS_FILENAME;
    let podcast_rss_filename = PODCAST_RSS_FILENAME;

    let t_plain_rss = translations.plain_rss;
    formatdoc!(r#"
        <link href="{atom_filename}" rel="alternate" title="Atom" type="application/atom+xml">
        <link href="{media_rss_filename}" rel="alternate" title="Media RSS" type="application/rss+xml">
        <link href="{plain_rss_filename}" rel="alternate" title="{t_plain_rss}" type="application/rss+xml">
        <link href="{podcast_rss_filename}" rel="alternate" title="Podcast RSS" type="application/rss+xml">
    "#)
}

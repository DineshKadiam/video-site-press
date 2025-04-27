// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Context,
    Site,
    SitePath,
    Translations
};
use crate::build::{AssetHashes, Layout};
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_button, unlisted_badge};
use crate::util::html_escape_outside_attribute;

use super::feeds::{
    ATOM_FILENAME,
    MEDIA_RSS_FILENAME,
    PLAIN_RSS_FILENAME,
    PODCAST_RSS_FILENAME
};

pub fn feed_choice(
    description: &str,
    filename: &str,
    label: &str,
    site: &Site,
    site_path: &SitePath,
    translations: &Translations
) -> String {
    let url = site.absolute_url_file_unchecked(filename, site_path);
    let copy_button = copy_button("content", &url, &translations.copy_link);

    formatdoc!(r#"
        <div>
            <h2>
                {label}
            </h2>
            <p>{description}</p>
            <a href="{url}">
                {url}
            </a>
            <div class="actions">
                {copy_button}
            </div>
        </div>
    "#)
}

/// Writes the page on which various feed subscription choices for a playlist
/// are made available.
pub fn subscribe_html(
    asset_hashes: &AssetHashes,
    context: &Context,
    site: &Site,
    site_path: &SitePath,
    subscribe_permalink: &str,
    title: &str,
    traversal: &Traversal,
    unlisted: bool
) {
    let mut layout = Layout::new();
    let translations = &site.language.translations;

    layout.add_clipboard_script();
    layout.no_indexing();

    let t_subscribe = translations.subscribe;
    let subscribe_title = format!("{t_subscribe} – {title}");
    let e_title = html_escape_outside_attribute(&subscribe_title);

    let r_unlisted_badge = if unlisted {
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let atom_feed_choice = feed_choice(
        translations.atom_description,
        ATOM_FILENAME,
        "Atom",
        site,
        site_path,
        translations
    );

    let media_rss_choice = feed_choice(
        translations.media_rss_description,
        MEDIA_RSS_FILENAME,
        "Media RSS",
        site,
        site_path,
        translations
    );

    let plain_rss_feed_choice = feed_choice(
        translations.plain_rss_description,
        PLAIN_RSS_FILENAME,
        &translations.plain_rss,
        site,
        site_path,
        translations
    );

    let podcast_rss_choice = feed_choice(
        translations.podcast_rss_description,
        PODCAST_RSS_FILENAME,
        "Podcast RSS",
        site,
        site_path,
        translations
    );

    let body = format!(r#"
        <div class="video_info">
            <h1>{e_title} {r_unlisted_badge}</h1>
            {atom_feed_choice}
            {media_rss_choice}
            {plain_rss_feed_choice}
            {podcast_rss_choice}
        </div>
    "#);

    let html = layout.render(
        asset_hashes,
        &body,
        site,
        &subscribe_title,
        traversal
    );

    let subscribe_dir = context.build_dir
        .join(site_path.filesystem_relative())
        .join(subscribe_permalink);

    fs::create_dir_all(&subscribe_dir).unwrap();
    let index_path = subscribe_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use hyper8_translations::Translations;
use hyper8_util::html_escape_outside_attribute;

use crate::{
    Build,
    Context,
    Site,
    SitePath
};
use crate::build::Layout;
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_button, unlisted_badge};

use super::feeds::{
    ATOM_FILENAME,
    MEDIA_RSS_FILENAME,
    PLAIN_RSS_FILENAME,
    PODCAST_RSS_FILENAME
};

pub fn feed_choice(
    description: &str,
    filename: &str,
    icon: &str,
    label: &str,
    site: &Site,
    site_path: &SitePath,
    translations: &Translations
) -> String {
    let url = site.absolute_url_file_unchecked(filename, site_path);
    let copy_button = copy_button("content", &url, translations.copy_url);

    formatdoc!(r#"
        <div class="feed_card">
            <h2>
                {icon}
                <span>{label}</span>
                <div class="actions">
                    {copy_button}
                </div>
            </h2>
            <a href="{url}">
                {url}
            </a>
            <p>{description}</p>
        </div>
    "#)
}

/// Writes the page on which various feed subscription choices for a playlist
/// are made available.
pub fn subscribe_html(
    build: &Build,
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
        hyper8_icons::ATOM,
        "Atom",
        site,
        site_path,
        translations
    );

    let media_rss_choice = feed_choice(
        translations.media_rss_description,
        MEDIA_RSS_FILENAME,
        &hyper8_icons::rss(None),
        "Media RSS",
        site,
        site_path,
        translations
    );

    let plain_rss_feed_choice = feed_choice(
        translations.plain_rss_description,
        PLAIN_RSS_FILENAME,
        &hyper8_icons::rss(None),
        translations.plain_rss,
        site,
        site_path,
        translations
    );

    let podcast_rss_choice = feed_choice(
        translations.podcast_rss_description,
        PODCAST_RSS_FILENAME,
        &hyper8_icons::rss(None),
        "Podcast RSS",
        site,
        site_path,
        translations
    );

    let body = format!(r#"
        <div class="video_info">
            <h1>{e_title} {r_unlisted_badge}</h1>
            <div class="feed_grid">
                {atom_feed_choice}
                {media_rss_choice}
                {plain_rss_feed_choice}
                {podcast_rss_choice}
            </div>
        </div>
    "#);

    let html = layout.render(
        build,
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

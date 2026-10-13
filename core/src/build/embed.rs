// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Build,
    Context,
    Playlist,
    Site,
    Video
};
use crate::build::{META_ROBOTS_NOINDEX_NOFOLLOW};
use crate::build::AssetHashes;
use crate::build::outline::Traversal;
use crate::build::player::{player, player_icon_templates};

use super::meta_generator;

/// Writes html for the embed itself, i.e. what is pulled in from external
/// site B through an iframe when they embed a video from site A.
pub fn embed_html(
    build: &Build,
    context: &Context,
    playlist: Option<&Playlist>,
    site: &Site,
    video: &Video,
    traversal: &Traversal
) {
    let video_prefix = "../";
    let player_rendered = player(
        true,
        playlist,
        site,
        video,
        video_prefix
    );

    let body = format!(r#"
        {player_rendered}
    "#);

    let html = embed_layout(build, &body, site, traversal);

    let embed_dir = context.build_dir
        .join(video.site_path.filesystem_relative())
        .join("embed");

    fs::create_dir_all(&embed_dir).unwrap();
    let index_path = embed_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

pub fn embed_layout(
    build: &Build,
    body: &str,
    site: &Site,
    traversal: &Traversal
) -> String {
    let meta_generator = meta_generator();
    let meta_robots_noindex_nofollow = META_ROBOTS_NOINDEX_NOFOLLOW;

    let embed_css_asset_hash = build.asset_hashes.embed_css.as_ref().unwrap();
    let embed_js_asset_hash = AssetHashes::EMBED_JS;
    let root_prefix = traversal.prefix_asc();
    let head = formatdoc!(r#"
        <head>
            <meta charset="utf-8">
            {meta_generator}
            {meta_robots_noindex_nofollow}
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <script defer src="{root_prefix}embed.js?{embed_js_asset_hash}"></script>
            <link rel="stylesheet" href="{root_prefix}embed.css?{embed_css_asset_hash}">
        </head>
    "#);

    let body = formatdoc!(r#"
        <body>
            {body}
        </body>
    "#);

    let lang_code = &site.language.code;
    let dir_attribute = if site.language.is_rtl() { r#"dir="rtl""# } else { "" };

    let templates = player_icon_templates(&site.language.translations);

    formatdoc!(r#"
        <!doctype html>
        <html {dir_attribute} lang="{lang_code}">
            {head}
            {body}
            {templates}
        </html>
    "#)
}

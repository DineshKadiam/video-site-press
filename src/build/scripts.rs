// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{Context, Site, SiteItem};
use crate::util::url_safe_hash_base64;

use super::AssetHashes;

pub fn generate_adaptive_theme_js(context: &Context) {
    fs::write(
        context.build_dir.join("adaptive_theme.js"),
        include_str!("assets/adaptive_theme.js")
    ).unwrap();
}

pub fn generate_clipboard_js(context: &Context) {
    fs::write(
        context.build_dir.join("clipboard.js"),
        include_str!("assets/clipboard.js")
    ).unwrap();
}

pub fn generate_embed_js(context: &Context) {
    fs::write(
        context.build_dir.join("embed.js"),
        include_str!("assets/embed.js")
    ).unwrap();
}

pub fn generate_search_js(
    asset_hashes: &mut AssetHashes,
    context: &Context,
    site: &Site
) {
    let translations = &site.language.translations;

    let mut items = Vec::new();

    let mut visit = |item: SiteItem| {
        let image;
        let kind;
        let title;
        let site_path;

        match item {
            SiteItem::Collection(collection) => {
                image = None;
                kind = "collection";
                title = collection.title_or_slug_or_generic_label(&translations);
                site_path = collection.site_path.normalized();
            }
            SiteItem::Playlist(playlist) => {
                image = None;
                kind = "playlist";
                title = playlist.title_or_slug_or_generic_label(&translations);
                site_path = playlist.site_path.normalized();
            }
            SiteItem::Video(video) => {
                image = video.poster
                    .as_ref()
                    .and_then(|poster| poster.assets.as_ref())
                    .map(|assets| assets.max_160.target_filename("poster"));
                kind = "video";
                title = video.title_or_slug_or_generic_label(&translations);
                site_path = video.site_path.normalized();
            }
        }

        let r_image = if let Some(file_name) = image {
            format!("image: '{file_name}',")
        } else {
            String::new()
        };

        let title_escaped = js_escape_inside_single_quoted_string(title);
        let r_item = formatdoc!(r#"
            {{
                kind: '{kind}',
                {r_image}
                title: '{title_escaped}',
                url: '{site_path}'
            }}
        "#);

        items.push(r_item);
    };

    site.walk_recursive(&mut visit);

    let r_items = items.join(",\n");

    let t_nothing_found_for_xxx = js_escape_inside_single_quoted_string(translations.nothing_found_for_xxx);
    let t_showing_xxx_results_for_xxx = js_escape_inside_single_quoted_string(translations.showing_xxx_results_for_xxx);
    let mut js = formatdoc!(r#"
        const SEARCH_JS_T = {{
            nothingFoundForXxx: query => '{t_nothing_found_for_xxx}'.replace('{{query}}', query),
            showingXxxResultsForXxx: (count, query) => '{t_showing_xxx_results_for_xxx}'.replace('{{count}}', count).replace('{{query}}', query)
        }};

        const SITE_ITEMS = [
            {r_items}
        ];
    "#);

    js.push_str(include_str!("assets/search.js"));

    asset_hashes.search_js = Some(url_safe_hash_base64(&js));

    fs::write(
        context.build_dir.join("search.js"),
        js
    ).unwrap();
}

pub fn generate_video_js(context: &Context) {
    fs::write(
        context.build_dir.join("video.js"),
        include_str!("assets/video.js")
    ).unwrap();
}

/// Escapes `'` as `\'` and `\` as `\\`
fn js_escape_inside_single_quoted_string(string: &str) -> String {
    string
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
}

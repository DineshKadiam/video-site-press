// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    Translations,
    Video
};
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

    fn video_js(translations: &Translations, video: &Video) -> String {
        let image = video.poster
            .as_ref()
            .and_then(|poster| poster.assets.as_ref())
            .map(|assets| assets.max_160.target_filename("poster"));
        let title = video.title_or_slug_or_generic_label(&translations);
        let site_path = video.site_path.normalized();

        let r_image = if let Some(file_name) = image {
            format!("image:'{file_name}',")
        } else {
            String::new()
        };

        let title_escaped = js_escape_inside_single_quoted_string(title);

        format!("{{{r_image}sitePath:'{site_path}',title:'{title_escaped}'}}")
    }

    fn playlist_js(playlist: &Playlist, translations: &Translations) -> String {
        let title = playlist.title_or_slug_or_generic_label(&translations);
        let site_path = playlist.site_path.normalized();

        let title_escaped = js_escape_inside_single_quoted_string(title);

        let videos = playlist.public_videos_in_playlist_order()
            .iter()
            .map(|video| video_js(translations, &video))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{sitePath:'{site_path}',title:'{title_escaped}',videos:[{videos}]}}")
    }

    fn collection_js(collection: &Collection, translations: &Translations) -> String {
        let title = collection.title_or_slug_or_generic_label(&translations);
        let site_path = collection.site_path.normalized();

        let title_escaped = js_escape_inside_single_quoted_string(title);

        let playlists = collection.public_playlists_desc_by_release_date()
            .iter()
            .map(|playlist| playlist_js(&playlist, translations))
            .collect::<Vec<String>>()
            .join(", ");

        let subcollections = collection.public_subcollections_desc_by_release_date()
            .iter()
            .map(|subcollection| collection_js(&subcollection, translations))
            .collect::<Vec<String>>()
            .join(", ");

        let videos = collection.public_videos_in_collection_video_order()
            .iter()
            .map(|video| video_js(translations, &video))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{\
            playlists:[{playlists}],\
            sitePath:'{site_path}',\
            subcollections:[{subcollections}],\
            title:'{title_escaped}',\
            videos:[{videos}]\
        }}")
    }

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let r_collection = collection_js(collection, translations);
            let site_content = format!("{{collection:{r_collection}}}");
            Some(site_content)
        }
        SiteContent::Empty => None,
        SiteContent::Invalid { .. } => None,
        SiteContent::Playlist(playlist) => {
            let r_playlist = playlist_js(playlist, translations);
            let site_content = format!("{{playlist:{r_playlist}}}");
            Some(site_content)
        }
        SiteContent::Video(_) => None
    };

    if let Some(site_content) = site_content {
        let t_nothing_found_for_xxx = js_escape_inside_single_quoted_string(translations.nothing_found_for_xxx);
        let t_showing_xxx_results_for_xxx = js_escape_inside_single_quoted_string(translations.showing_xxx_results_for_xxx);
        let mut js = formatdoc!(r#"
            const SEARCH_JS_T = {{
                nothingFoundForXxx: query => '{t_nothing_found_for_xxx}'.replace('{{query}}', query),
                showingXxxResultsForXxx: (count, query) => '{t_showing_xxx_results_for_xxx}'.replace('{{count}}', count).replace('{{query}}', query)
            }};
            const siteContent = {site_content};
        "#);

        js.push_str(include_str!("assets/search.js"));

        asset_hashes.search_js = Some(url_safe_hash_base64(&js));

        fs::write(
            context.build_dir.join("search.js"),
            js
        ).unwrap();
    }
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

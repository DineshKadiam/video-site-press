// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Language,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::util::{
    casual_duration,
    localized_date_d_mmm_yyyy,
    url_safe_hash_base64
};

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

pub fn generate_navigation_js(
    asset_hashes: &mut AssetHashes,
    context: &Context,
    site: &Site
) {
    let language = &site.language;
    let translations = &language.translations;

    fn video_js(language: &Language, video: &Video) -> String {
        let image;
        let lqip;
        match &video.poster {
            Some(poster) => {
                image = if let Some(assets) = &poster.assets {
                    let target_filename = assets.max_160.target_filename("poster");
                    format!("image:'{target_filename}',")
                } else {
                    String::new()
                };

                lqip = if let Some(poster_meta) = &poster.poster_meta {
                    format!("lqip:{},", poster_meta.lqip.js_array())
                } else {
                    String::new()
                };
            }
            None => {
                image = String::new();
                lqip = String::new();
            }
        }

        let date = match video.release_date {
            Some(release_date) => {
                let date = localized_date_d_mmm_yyyy(release_date, language.chrono_locale);
                format!("date:'{date}',")
            }
            None => String::new()
        };

        let duration = match &video
            .highest_quality_video_file()
            .and_then(|video_file| video_file.video_meta.as_ref()) {
            Some(video_meta) => {
                let duration = casual_duration(video_meta.duration, &language.translations);
                format!("duration:'{duration}',")
            }
            None => String::new()
        };

        let site_path = video.site_path.normalized();
        let title = video.title_or_slug_or_generic_label(&language.translations);
        let title_escaped = js_escape_inside_single_quoted_string(title);

        format!("{{{date}{duration}{lqip}{image}sitePath:'{site_path}',title:'{title_escaped}'}}")
    }

    fn playlist_js(language: &Language, playlist: &Playlist) -> String {
        let title = playlist.title_or_slug_or_generic_label(&language.translations);
        let site_path = playlist.site_path.normalized();

        let title_escaped = js_escape_inside_single_quoted_string(title);

        let videos = playlist.public_videos_in_playlist_order(language)
            .iter()
            .map(|video| video_js(language, &video))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{sitePath:'{site_path}',title:'{title_escaped}',videos:[{videos}]}}")
    }

    fn collection_js(collection: &Collection, language: &Language) -> String {
        let title = collection.title_or_slug_or_generic_label(&language.translations);
        let site_path = collection.site_path.normalized();

        let title_escaped = js_escape_inside_single_quoted_string(title);

        let playlists = collection.public_playlists_desc_by_release_date()
            .iter()
            .map(|playlist| playlist_js(language, &playlist))
            .collect::<Vec<String>>()
            .join(", ");

        let subcollections = collection.public_subcollections_desc_by_release_date()
            .iter()
            .map(|subcollection| collection_js(&subcollection, language))
            .collect::<Vec<String>>()
            .join(", ");

        let videos = collection.public_videos_in_collection_video_order(language)
            .iter()
            .map(|video| video_js(language, &video))
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
            let r_collection = collection_js(collection, language);
            let site_content = format!("{{collection:{r_collection}}}");
            Some(site_content)
        }
        SiteContent::Empty => None,
        SiteContent::Invalid { .. } => None,
        SiteContent::Playlist(playlist) => {
            let r_playlist = playlist_js(language, playlist);
            let site_content = format!("{{playlist:{r_playlist}}}");
            Some(site_content)
        }
        SiteContent::Video(_) => None
    };

    if let Some(site_content) = site_content {
        let t_collapse = translations.collapse;
        let t_expand = translations.expand;
        let t_nothing_found_for_xxx = js_escape_inside_single_quoted_string(translations.nothing_found_for_xxx);
        let t_showing_xxx_results_for_xxx = js_escape_inside_single_quoted_string(translations.showing_xxx_results_for_xxx);
        let t_xxx_videos = js_escape_inside_single_quoted_string(translations.xxx_videos);
        let mut js = formatdoc!(r#"
            const NAVIGATION_JS_T = {{
                collapse: '{t_collapse}',
                expand: '{t_expand}',
                nothingFoundForXxx: query => '{t_nothing_found_for_xxx}'.replace('{{query}}', query),
                showingXxxResultsForXxx: (count, query) => '{t_showing_xxx_results_for_xxx}'.replace('{{count}}', count).replace('{{query}}', query),
                xxxVideos: count => '{t_xxx_videos}'.replace('{{count}}', count)
            }};
            const siteContent = {site_content};
        "#);

        js.push_str(include_str!("assets/navigation.js"));

        asset_hashes.navigation_js = Some(url_safe_hash_base64(&js));

        fs::write(
            context.build_dir.join("navigation.js"),
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

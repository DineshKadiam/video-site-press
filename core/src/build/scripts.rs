// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use hyper8_util::{casual_duration, url_safe_hash_base64};

use crate::{
    Build,
    Collection,
    Container,
    Context,
    Language,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::util::localized_date_d_mmm_yyyy;

pub fn generate_adaptive_theme_js(context: &Context) {
    let js = scope_js(include_str!("assets/adaptive_theme.js"));
    let path = context.build_dir.join("adaptive_theme.js");

    fs::write(path, js).unwrap();
}

pub fn generate_clipboard_js(context: &Context) {
    let js = scope_js(include_str!("assets/clipboard.js"));
    let path = context.build_dir.join("clipboard.js");

    fs::write(path, js).unwrap();
}

pub fn generate_embed_js(context: &Context) {
    let js = scope_js(include_str!("assets/embed.js"));
    let path = context.build_dir.join("embed.js");

    fs::write(path, js).unwrap();
}

pub fn generate_navigation_js(
    build: &mut Build,
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

        let duration = match video.duration_public() {
            Some(duration) => {
                let duration_formatted = casual_duration(duration, &language.translations);
                format!("duration:'{duration_formatted}',")
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
            .map(|video| video_js(language, video))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{sitePath:'{site_path}',title:'{title_escaped}',videos:[{videos}]}}")
    }

    fn collection_js(collection: &Collection, language: &Language) -> String {
        let title = collection.title_or_slug_or_generic_label(&language.translations);
        let site_path = collection.site_path.normalized();

        let title_escaped = js_escape_inside_single_quoted_string(title);

        let containers = collection.public_containers_in_container_order(language)
            .iter()
            .map(|container| {
                match container {
                    Container::Collection(collection) => collection_js(collection, language),
                    Container::Playlist(playlist) => playlist_js(language, playlist)
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        let videos = collection.public_videos_in_collection_video_order(language)
            .iter()
            .map(|video| video_js(language, video))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{{\
            containers:[{containers}],\
            sitePath:'{site_path}',\
            title:'{title_escaped}',\
            videos:[{videos}]\
        }}")
    }

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            Some(collection_js(collection, language))
        }
        SiteContent::Empty => None,
        SiteContent::Invalid { .. } => None,
        SiteContent::Playlist(playlist) => {
            Some(playlist_js(language, playlist))
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
            const siteContent = {site_content};
            const t = {{
                collapse: '{t_collapse}',
                expand: '{t_expand}',
                nothingFoundForXxx: query => '{t_nothing_found_for_xxx}'.replace('{{query}}', query),
                showingXxxResultsForXxx: (count, query) => '{t_showing_xxx_results_for_xxx}'.replace('{{count}}', count).replace('{{query}}', query),
                xxxVideos: count => '{t_xxx_videos}'.replace('{{count}}', count)
            }};
        "#);

        js.push_str(&scope_js(include_str!("assets/navigation.js")));

        build.asset_hashes.navigation_js = Some(url_safe_hash_base64(js.as_bytes()));

        let path = context.build_dir.join("navigation.js");

        fs::write(path, js).unwrap();
    }
}

pub fn generate_video_js(context: &Context) {
    let js = scope_js(include_str!("assets/video.js"));
    let path = context.build_dir.join("video.js");

    fs::write(path, js).unwrap();
}

/// Escapes `'` as `\'` and `\` as `\\`
fn js_escape_inside_single_quoted_string(string: &str) -> String {
    string
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
}

/// We do not use ESM (modules) on generated Hyper 8 sites for any of
/// the scripts, because doing so would take away the possibility to
/// open and use Hyper 8 sites directly from disk (file:// protocol).
/// As a consequence, all our scripts operate in the global (window)
/// scope, and we need to wrap them in IIFEs to avoid potential name
/// collisions (e.g. two scripts might define "const exampleButton = (...)"
/// because they both need to reference the same button).
pub fn scope_js(js: &str) -> String {
    format!("(()=>{{{js}}})();")
}

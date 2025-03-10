// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use indoc::formatdoc;

use crate::{
    Banner,
    Language,
    Link,
    PosterFile,
    Site,
    Translations,
    Video,
    VideoFile
};
use crate::build::{AssetHashes, Traversal};
use crate::icons;
use crate::util::{
    casual_duration,
    html_escape_outside_attribute,
    human_size,
    precise_duration
};

use super::{META_GENERATOR, META_ROBOTS_NOINDEX_NOFOLLOW};

pub fn banner_html(banner: &Banner, prefix: &str) -> String {
    let alt = if let Some(description) = &banner.description {
        format!(r#"alt="{description}""#)
    } else {
        String::new()
    };

    if let Some(poster_assets) = &banner.assets {
        // TODO: Adjust up_to_xxx (depends on final content width we're using)
        let image_attributes = poster_assets.img_attributes_up_to_1280("banner", prefix);
        let src = image_attributes.src;
        let srcset = image_attributes.srcset;

        formatdoc!(r#"<img {alt} class="banner" src="{src}" srcset="{srcset}">"#)
    } else {
        formatdoc!(r#"<img {alt} class="banner" src="{prefix}banner.jpg">"#)
    }
}

/// A button enriched with data attributes that client scripting can use
/// to copy the content (embed code or link) to clipboard and display success/failure state.
/// Returns a tuple `(button, templates)` where `button` contains the markup
/// for the copy link button itself, and `templates` the markup for three
/// template tags that are only used at runtime by the clipboard script to
/// switch the button's icon for visual feedback. The template tags should
/// only be appended at the bottom of the page to optimize page load.
pub fn copy_button(content_key: &str, content_value: &str, label: &str, site: &Site) -> (String, String) {
    let copy_icon = icons::copy();
    let failure_icon = icons::failure(site.language.translations.failed);
    let success_icon = icons::success(site.language.translations.copied);

    let button = format!(r#"
        <button data-{content_key}="{content_value}" data-copy>
            <span class="icon">{copy_icon}</span>
            <span>{label}</label>
        </button>
    "#);

    let templates = format!(r#"
        <template id="copy_icon">
            {copy_icon}
        </template>
        <template id="failure_icon">
            {failure_icon}
        </template>
        <template id="success_icon">
            {success_icon}
        </template>
    "#);

    (button, templates)
}

pub fn embed_layout(
    asset_hashes: &AssetHashes,
    body: &str,
    site: &Site,
    traversal: &Traversal
) -> String {
    let meta_generator = META_GENERATOR;
    let meta_robots_noindex_nofollow = META_ROBOTS_NOINDEX_NOFOLLOW;

    let embed_css_asset_hash = asset_hashes.embed_css.as_ref().unwrap();
    let embed_js_asset_hash = AssetHashes::EMBED_JS;
    let root_prefix = traversal.prefix_asc();
    let head = formatdoc!(r#"
        <head>
            <meta charset="utf-8">
            {meta_generator}
            {meta_robots_noindex_nofollow}
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <script defer src="{root_prefix}embed.js?{embed_js_asset_hash}" type="module"></script>
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

    formatdoc!(r#"
        <!doctype html>
        <html {dir_attribute} lang="{lang_code}">
            {head}
            {body}
        </html>
    "#)
}

pub fn links(language: &Language, links: &[Link]) -> String {
    if links.is_empty() {
        return String::new();
    }

    let external_icon = icons::external(language.translations.external_link);

    let items = links
        .iter()
        .map(|link| {
            let rel_me = if link.rel_me { r#"rel="me""# } else { "" };
            let url = &link.url;

            if link.hidden {
                format!(r#"<a href="{url}" {rel_me} style="display: none;">hidden</a>"#)
            } else {
                let label = link.pretty_label();
                let e_label = html_escape_outside_attribute(&label);
                formatdoc!(r#"
                    <a href="{url}" {rel_me} target="_blank">{external_icon} <span>{e_label}</span></a>
                "#)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"<div class="links">{items}</div>"#)
}

/// Renders the video player.
/// In the context of a playlist, [next_video] references the next video
/// in the playlist, if there is one.
pub fn player(
    next_video: Option<Arc<Video>>,
    site: &Site,
    video: &Video,
    video_prefix: &str
) -> String {
    let index_suffix = site.index_suffix();

    if video.versions.is_empty() {
        // TODO: Should use poster assets (different sizes)
        let poster = match &video.poster {
            Some(poster) => {
                let alt = if let Some(description) = &poster.description {
                    format!(r#"alt="{description}" "#)
                } else {
                    String::new()
                };

                format!(r#"<img {alt}class="thumbnail" src="{video_prefix}poster.jpg">"#)
            }
            None => String::new()
        };

        let t_video_not_yet_available = site.language.translations.video_not_yet_available;

        return formatdoc!(r#"
            <div style="border: 1px solid red; height: 12.8rem; width: 19.2rem;">
                {poster}
                {t_video_not_yet_available}
            </div>
        "#);
    }

    // TODO: This fails on preview build if we don't have metadata yet (or something like that)
    let recommended_version = video.recommended_version().unwrap();

    let poster = match &video.poster {
        Some(poster) => match &poster.assets {
            Some(poster_assets) => {
                let largest = poster_assets.largest();
                let height = largest.height;
                let width = largest.width;
                format!(r#"poster="{video_prefix}poster_{width}x{height}.jpg""#)
            }
            None => format!(r#"poster="{video_prefix}poster.jpg""#)
        }
        None => String::new()
    };

    let subtitles = video.subtitles
        .iter()
        .enumerate()
        .map(|(index, subtitle_file)| {
            // TODO: label needs to be escaped
            let (label, srclang) = match &subtitle_file.language {
                Some(code) => (code, format!(r#"srclang="{code}""#)),
                None => (&subtitle_file.file_name, String::new())
            };

            // TODO: Dynamic sourcing from user-supplied data for kind/label
            // TODO: Maybe use 'default' attribute (but based on what?)
            formatdoc!(r#"
                <track
                    kind="subtitles"
                    label="{label}"
                    src="{video_prefix}{index}.vtt"
                    {srclang}/>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let subtitles_icon = icons::subtitles(site.language.translations.subtitles);
    let subtitle_select = match video.subtitles.is_empty() {
        true => String::new(),
        false => format!(r#"<button id="select_subtitles">{subtitles_icon}</button>"#)
    };

    let duration = recommended_version.video_meta.as_ref().unwrap().duration;
    let duration_formatted = precise_duration(duration);
    let default_resolution_label = recommended_version.vertical_resolution_label();
    let src = &recommended_version.file_name;

    let versions_ring_rendered = versions_ring(recommended_version, video, video_prefix);

    let t_next_video = site.language.translations.next_video;
    let next_video_button = match next_video {
        Some(next_video) => format!(r#"<a href="{video_prefix}../{}{index_suffix}">{t_next_video}</a>"#, next_video.slug_unchecked()),
        None => String::new()
    };

    let maximize_icon = icons::maximize(site.language.translations.maximize);
    let minimize_icon = icons::minimize(site.language.translations.minimize);
    let pause_icon = icons::pause(site.language.translations.pause);
    let play_icon = icons::play(site.language.translations.play);
    let volume_loud_icon = icons::volume_loud(site.language.translations.loud_volume);
    let volume_medium_icon = icons::volume_medium(site.language.translations.medium_volume);
    let volume_muted_icon = icons::volume_muted(site.language.translations.sound_muted);
    let t_replay = site.language.translations.replay;
    formatdoc!(r#"
        <div class="player_wrapper">
            <div class="player_intro">
                {versions_ring_rendered}
            </div>
            <div class="player_outro">
                <button class="align flex space" id="replay">
                    {play_icon}
                    {t_replay}
                </button>
                {next_video_button}
            </div>
            <div class="player_controls">
                <div class="timeline">
                    <div class="bar backdrop"></div>
                    <div class="bar progress"></div>
                    <input aria-valuenow="0" aria-valuetext="00:00" max="{duration}" min="0" step="any" type="range" value="0">
                    <div class="bar outline"></div>
                </div>
                <div class="control_pane">
                    <div>
                        <button id="toggle_playback">{play_icon}</button>
                        <button id="toggle_volume">{volume_loud_icon}</button>
                        <button id="toggle_speed">1×</button>
                        <div>
                            <span id="time_current">00:00</span> / <span id="time_total">{duration_formatted}</span>
                        </div>
                    </div>
                    <div>
                        <button id="select_resolution">{default_resolution_label}</button>
                        {subtitle_select}
                        <button id="toggle_fullscreen">{maximize_icon}</button>
                    </div>
                </div>
            </div>
            <video data-duration="{duration}" playsinline {poster} preload="none" src="{video_prefix}{src}">
                {subtitles}
            </video>
            <template id="maximize_icon">
                {maximize_icon}
            </template>
            <template id="minimize_icon">
                {minimize_icon}
            </template>
            <template id="pause_icon">
                {pause_icon}
            </template>
            <template id="play_icon">
                {play_icon}
            </template>
            <template id="subtitles_icon">
                {subtitles_icon}
            </template>
            <template id="volume_loud_icon">
                {volume_loud_icon}
            </template>
            <template id="volume_medium_icon">
                {volume_medium_icon}
            </template>
            <template id="volume_muted_icon">
                {volume_muted_icon}
            </template>
        </div>
    "#)
}

pub fn poster_html(poster: &PosterFile, video_prefix: &str) -> String {
    let alt = if let Some(description) = &poster.description {
        format!(r#"alt="{description}""#)
    } else {
        String::new()
    };

    match &poster.assets {
        Some(poster_assets) => {
            let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);
            let src = image_attributes.src;
            let srcset = image_attributes.srcset;

            format!(r#"<img {alt} src="{src}" srcset="{srcset}">"#)
        }
        None => {
            format!(r#"<img {alt} src="{video_prefix}poster.jpg">"#)
        }
    }
}

pub fn thumbnail(
    translations: &Translations,
    video: &Video,
    video_prefix: &str,
    with_duration: bool
) -> String {
    match &video.poster {
        Some(poster) => {
            let alt = if let Some(description) = &poster.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            let img = match &poster.assets {
                Some(poster_assets) => {
                    let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);
                    let src = image_attributes.src;
                    let srcset = image_attributes.srcset;

                    format!(r#"<img {alt} class="thumbnail" src="{src}" srcset="{srcset}">"#)
                }
                None => {
                    format!(r#"<img {alt} class="thumbnail" src="{video_prefix}poster.jpg">"#)
                }
            };

            if !with_duration {
                return img;
            }

            let duration = match &video.highest_quality_version().and_then(|version| version.video_meta.as_ref()) {
                Some(video_meta) => {
                    let duration = casual_duration(video_meta.duration, translations);
                    formatdoc!(r#"<span class="duration">{duration}</span>"#)
                }
                None => String::new()
            };

            formatdoc!(r#"
                <div class="thumbnail_wrapper">
                    {img}
                    {duration}
                </div>
            "#)
        },
        None => String::new()
    }
}

pub fn unlisted_badge(translations: &Translations) -> String {
    let t_not_public = translations.not_public;
    format!(r#"<span class="unlisted_badge">{t_not_public}</span>"#)
}

fn versions_ring(
    recommended_version: &VideoFile,
    video: &Video,
    video_prefix: &str
) -> String {
    let radius = 190.0;
    let diameter = radius * 2.0;

    let mut sizes = Vec::new();
    let mut total_size: u64 = 0;

    let mut largest_size = 0;

    for version in &video.versions {
        let file_name = &version.file_name;
        let size = version.file_meta.size;
        let resolution = version.vertical_resolution_label();
        sizes.push((file_name, resolution, size));
        total_size += size;

        if size > largest_size {
            largest_size = size;
        }
    }

    sizes.sort_unstable_by(|a, b| a.2.cmp(&b.2));

    // Draws a ring segment. In clock terms we start at 12 o'clock and we go clockwise.
    let slice = |
        arc_angle: f32,
        begin_angle: f32,
        default: bool,
        resolution: &str,
        size: &str,
        src: &str
    | -> String {
        let large_arc_flag = if arc_angle < 180.0 { 0 } else { 1 };

        let begin_angle_rad = begin_angle.to_radians();
        let begin_x = begin_angle_rad.sin();
        let begin_y = -begin_angle_rad.cos();

        let end_angle_rad = (begin_angle + arc_angle).to_radians();
        let end_x = end_angle_rad.sin();
        let end_y = -end_angle_rad.cos();

        let outer_radius = radius * 0.5;
        let segment_outer_begin_x = radius + begin_x * outer_radius;
        let segment_outer_begin_y = radius + begin_y * outer_radius;

        let segment_outer_end_x = radius + end_x * outer_radius;
        let segment_outer_end_y = radius + end_y * outer_radius;

        let inner_radius = radius * 0.4;
        let segment_inner_begin_x = radius + begin_x * inner_radius;
        let segment_inner_begin_y = radius + begin_y * inner_radius;

        let segment_inner_end_x = radius + end_x * inner_radius;
        let segment_inner_end_y = radius + end_y * inner_radius;

        let text_radius = radius * 0.8;
        let text_spacing = radius * 0.12;
        let text_angle_rad = (begin_angle + 0.5 * arc_angle).to_radians();
        let text_x = radius + text_angle_rad.sin() * text_radius;
        let text_resolution_y = radius + -text_angle_rad.cos() * text_radius;
        let text_size_y = radius + -text_angle_rad.cos() * text_radius + text_spacing;

        let play_button = if default {
            let ratio = 1.197655;
            let width = radius * 0.2;
            let height = ratio * width;
            let a_x = radius - width * 0.5;
            let a_y = radius - height * 0.5;
            let b_x = radius + width * 0.5;
            let b_y = radius;
            let c_x = radius - width * 0.5;
            let c_y = radius + height * 0.5;
            formatdoc!(r#"
                <circle cx="{radius}" cy="{radius}" fill="transparent" r="{inner_radius}"/>
                <path d="M {a_x},{a_y} L {b_x},{b_y} L {c_x},{c_y} Z"/>
            "#)
        } else {
            String::new()
        };

        formatdoc!(r#"
                <g
                    data-label="{resolution}"
                    data-src="{src}"
                    tabindex="0">
                    {play_button}
                    <path d="M {segment_outer_begin_x},{segment_outer_begin_y}
                           A {outer_radius} {outer_radius} 0 {large_arc_flag} 1 {segment_outer_end_x},{segment_outer_end_y}
                           L {segment_inner_end_x},{segment_inner_end_y}
                           A {inner_radius} {inner_radius} 0 {large_arc_flag} 0 {segment_inner_begin_x},{segment_inner_begin_y}
                           Z">
                    </path>
                    <text text-anchor="middle" x="{text_x}" y="{text_resolution_y}">
                        {resolution}
                    </text>
                    <text class="size" text-anchor="middle" x="{text_x}" y="{text_size_y}">
                        {size}
                    </text>
                    <title>{resolution} {size}</title>
                </g>
        "#)
    };

    let mut slices = String::new();
    let gap_angle = 2.5;
    let usable_angle = 360.0 - sizes.len() as f32 * gap_angle;
    let mut begin_angle = gap_angle * 0.5;

    for (file_name, resolution, size) in sizes.iter() {
        let arc_angle = if sizes.len() == 1 {
            usable_angle
        } else {
            (*size as f32 / total_size as f32) * usable_angle
        };

        let size_humanized = human_size(*size);

        let default = &recommended_version.file_name == *file_name;
        let src = format!("{video_prefix}{file_name}");
        let slice_rendered = slice(
            arc_angle,
            begin_angle,
            default,
            resolution,
            &size_humanized,
            &src
        );
        slices.push_str(&slice_rendered);

        begin_angle += arc_angle + gap_angle;
    }

    formatdoc!(r#"
        <svg class="versions_ring" width="{diameter}" height="{diameter}" version="1.1" viewBox="0 0 {diameter} {diameter}" xmlns="http://www.w3.org/2000/svg">
            {slices}
        </svg>
    "#)
}

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

/// A button enriched with data attributes that client scripting can use to
/// copy the content (embed code or link) to clipboard and display
/// success/failure state.
pub fn copy_button(
    content_key: &str,
    content_value: &str,
    label: &str
) -> String {
    let copy_icon = icons::COPY;
    format!(r#"
        <button data-{content_key}="{content_value}" data-copy>
            <span class="icon">{copy_icon}</span>
            <span>{label}</label>
        </button>
    "#)
}

/// <template> tags to be used in client-side (browser) scripting at runtime,
/// these are injected to the end of the body in the regular page and embed
/// layouts.
pub fn copy_button_icon_templates(translations: &Translations) -> String {
    let copy_icon = icons::COPY;
    let failure_icon = icons::failure(translations.failed);
    let success_icon = icons::success(translations.copied);

    formatdoc!(r#"
        <template id="copy_icon">
            {copy_icon}
        </template>
        <template id="failure_icon">
            {failure_icon}
        </template>
        <template id="success_icon">
            {success_icon}
        </template>
    "#)
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
    let translations = &site.language.translations;

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

        let t_video_not_yet_available = translations.video_not_yet_available;

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

    let subtitles_icon = icons::subtitles(translations.subtitles);
    let subtitle_select = match video.subtitles.is_empty() {
        true => String::new(),
        false => format!(r#"<button id="select_subtitles">{subtitles_icon}</button>"#)
    };

    let duration = recommended_version.video_meta.as_ref().unwrap().duration;
    let duration_formatted = precise_duration(duration);
    let default_resolution_label = recommended_version.vertical_resolution_label();
    let src = &recommended_version.file_name;

    let versions_ring_rendered = versions_ring(recommended_version, video, video_prefix);

    let t_next_video = translations.next_video;
    let next_video_button = match next_video {
        Some(next_video) => format!(r#"<a href="{video_prefix}../{}{index_suffix}">{t_next_video}</a>"#, next_video.slug_unchecked()),
        None => String::new()
    };

    let maximize_icon = icons::maximize(translations.maximize);
    let play_icon = icons::play(translations.play);
    let volume_loud_icon = icons::volume_loud(translations.loud_volume);
    let t_replay = translations.replay;
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
                        <button id="speed"><span class="multiplier">1.0</span><span class="x">x</span></button>
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
        </div>
    "#)
}

/// <template> tags to be used in client-side (browser) scripting at runtime,
/// these are injected to the end of the body in the regular page and embed
/// layouts.
pub fn player_icon_templates(translations: &Translations) -> String {
    let maximize_icon = icons::maximize(translations.maximize);
    let minimize_icon = icons::minimize(translations.minimize);
    let pause_icon = icons::pause(translations.pause);
    let play_icon = icons::play(translations.play);
    let subtitles_icon = icons::subtitles(translations.subtitles);
    let volume_loud_icon = icons::volume_loud(translations.loud_volume);
    let volume_medium_icon = icons::volume_medium(translations.medium_volume);
    let volume_muted_icon = icons::volume_muted(translations.sound_muted);

    formatdoc!(r#"
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

/// Assumes that subscribe_permalink is a subfolder of the current location,
/// i.e. we render it from the perspective of /foo/bar/ and the subscribe
/// page is assumed to be at /foo/bar/[subscribe_permalink]/.
pub fn subscribe_button(
    subscribe_permalink: &str,
    site: &Site
) -> String {
    let index_suffix = site.index_suffix();
    let feed_icon = icons::rss(Some(site.language.translations.feed));
    let t_subscribe = site.language.translations.subscribe;

    formatdoc!(r#"
        <div class="actions flex">
            <a href="{subscribe_permalink}{index_suffix}">
                {feed_icon} {t_subscribe}
            </a>
        </div>
    "#)
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
    const RADIUS: f32 = 190.0;
    const DIAMETER: f32 = RADIUS * 2.0;
    const GAP_ANGLE: f32 = 2.5;

    // Radii between which the slices of the ring are placed
    const INNER_RADIUS: f32 = RADIUS * 0.39;
    const OUTER_RADIUS: f32 = RADIUS * 0.5;

    const TEXT_RADIUS: f32 = RADIUS * 0.8;
    const TEXT_SPACING: f32 = RADIUS * 0.12;

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

    // Draws a ring segment - we start at 12 o'clock and go clockwise.
    let slice = |
        arc_angle: f32,
        begin_angle: f32,
        default: bool,
        resolution: &str,
        size: &str,
        src: &str
    | -> String {
        // Our specific drawing approach for a slice does not support drawing
        // a mathematically perfect 360 degree slice, therefore we simulate
        // it with a (visually indistinguishable) almost 360 degree slice.
        let safe_arc_angle = if arc_angle < 360.0 { arc_angle } else { 359.999 };

        let slice = {
            let large_arc_flag = if safe_arc_angle < 180.0 { 0 } else { 1 };

            let begin_angle_rad = begin_angle.to_radians();
            let begin_x = begin_angle_rad.sin();
            let begin_y = -begin_angle_rad.cos();

            let end_angle_rad = (begin_angle + safe_arc_angle).to_radians();
            let end_x = end_angle_rad.sin();
            let end_y = -end_angle_rad.cos();

            let segment_outer_begin_x = RADIUS + begin_x * OUTER_RADIUS;
            let segment_outer_begin_y = RADIUS + begin_y * OUTER_RADIUS;

            let segment_outer_end_x = RADIUS + end_x * OUTER_RADIUS;
            let segment_outer_end_y = RADIUS + end_y * OUTER_RADIUS;

            let segment_inner_begin_x = RADIUS + begin_x * INNER_RADIUS;
            let segment_inner_begin_y = RADIUS + begin_y * INNER_RADIUS;

            let segment_inner_end_x = RADIUS + end_x * INNER_RADIUS;
            let segment_inner_end_y = RADIUS + end_y * INNER_RADIUS;

            formatdoc!(r#"
                <path d="M {segment_outer_begin_x},{segment_outer_begin_y}
                         A {OUTER_RADIUS} {OUTER_RADIUS} 0 {large_arc_flag} 1 {segment_outer_end_x},{segment_outer_end_y}
                         L {segment_inner_end_x},{segment_inner_end_y}
                         A {INNER_RADIUS} {INNER_RADIUS} 0 {large_arc_flag} 0 {segment_inner_begin_x},{segment_inner_begin_y}
                         Z"/>
            "#)
        };

        let text_angle_rad = (begin_angle + 0.5 * safe_arc_angle).to_radians();
        let text_x = RADIUS + text_angle_rad.sin() * TEXT_RADIUS;
        let text_resolution_y = RADIUS + -text_angle_rad.cos() * TEXT_RADIUS;
        let text_size_y = RADIUS + -text_angle_rad.cos() * TEXT_RADIUS + TEXT_SPACING;

        let play_button = if default {
            const PLAY_BUTTON_RATIO: f32 = 1.197655;
            const PLAY_BUTTON_WIDTH: f32 = RADIUS * 0.2;
            const PLAY_BUTTON_HEIGHT: f32 = PLAY_BUTTON_RATIO * PLAY_BUTTON_WIDTH;
            let a_x = RADIUS - PLAY_BUTTON_WIDTH * 0.5;
            let a_y = RADIUS - PLAY_BUTTON_HEIGHT * 0.5;
            let b_x = RADIUS + PLAY_BUTTON_WIDTH * 0.5;
            let b_y = RADIUS;
            let c_x = RADIUS - PLAY_BUTTON_WIDTH * 0.5;
            let c_y = RADIUS + PLAY_BUTTON_HEIGHT * 0.5;
            formatdoc!(r#"
                <circle cx="{RADIUS}" cy="{RADIUS}" fill="transparent" r="{INNER_RADIUS}"/>
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
                {slice}
                <rect class="backdrop" filter="url(#text_shadow)" rx="5"/>
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

    let mut begin_angle = if sizes.len() == 1 {
        0.0
    } else if sizes.len() == 2 {
        // With two slices we place them so that both slice midpoints lie
        // exactly on the horziontal middle line of the ring, i.e. one
        // (the bigger) version is exactly left, and the other one exactly
        // right on the version ring.
        90.0 - (sizes[0].2 as f32 / total_size as f32) * 180.0 - GAP_ANGLE * 0.5
    } else {
        // With 2+ slices we begin at 12 o'clock and then draw the slices
        // in ascending size, going clockwise on the ring.
        GAP_ANGLE * 0.5
    };

    for (file_name, resolution, size) in sizes.iter() {
        let arc_angle = if sizes.len() == 1 {
            360.0
        } else {
            (*size as f32 / total_size as f32) * (360.0 - sizes.len() as f32 * GAP_ANGLE)
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

        begin_angle += arc_angle + GAP_ANGLE;
    }

    const BLUR_DEVIATION: usize = 8;
    const BLUR_Y_OFFSET: f32 = BLUR_DEVIATION as f32 / 2.0;
    const BACKDROP_Y: f32 = RADIUS + BLUR_Y_OFFSET;
    const BACKDROP_RADIUS: f32 = OUTER_RADIUS - (BLUR_DEVIATION as f32 / 2.0);

    formatdoc!(r#"
        <svg class="versions_ring" width="{DIAMETER}" height="{DIAMETER}" version="1.1" viewBox="0 0 {DIAMETER} {DIAMETER}" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <filter id="ring_shadow">
                    <feGaussianBlur in="SourceGraphic" stdDeviation="{BLUR_DEVIATION}" />
                </filter>
                <filter id="text_shadow">
                    <feDropShadow dx="0" dy="3" flood-color="var(--d-2)" flood-opacity="0.4" stdDeviation="3"/>
                </filter>
            </defs>
            <circle class="backdrop" cx="{RADIUS}" cy="{BACKDROP_Y}" filter="url(#ring_shadow)" r="{BACKDROP_RADIUS}"/>
            {slices}
        </svg>
    "#)
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use indoc::formatdoc;

use crate::{
    AudioFile,
    Playlist,
    Site,
    Translations,
    Video,
    VideoFile
};
use crate::build::widgets::thumbnail;
use crate::icons;
use crate::util::{
    casual_duration,
    html_escape_inside_attribute,
    html_escape_outside_attribute,
    human_size,
    localized_date_d_mmm_yyyy,
    precise_duration
};

/// Renders the video player.
pub fn player(
    // In the embedded player, the intro screen has additional
    // (redundant) volume/subtitles/fullscreen controls, next to the
    // resolution selector.
    embed: bool,
    playlist: Option<&Playlist>,
    site: &Site,
    video: &Video,
    video_prefix: &str
) -> String {
    let language = &site.language;
    let translations = &language.translations;

    if video.audio_files.is_empty() && video.video_files.is_empty() {
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

    let recommended_video_file = video.recommended_video_file();

    let poster = match &video.poster {
        Some(poster) => {
            // TODO: Compute, cache and use a compressed/normalized (high-res, but
            // still constrained to a maximum resolution) version of the poster
            // for use here
            let original_extension = Path::new(&poster.file_name)
                .extension()
                .unwrap()
                .to_string_lossy();

            format!(r#"poster="{video_prefix}poster.{original_extension}""#)
        }
        None => String::new()
    };

    let active_marker_icon = icons::active_marker(translations.active);

    let subtitle_controls;
    let subtitle_tracks;
    if !video.subtitles.is_empty() {
        subtitle_tracks = video.subtitles
            .iter()
            .enumerate()
            .map(|(index, subtitle_file)| {
                let label = html_escape_inside_attribute(&subtitle_file.label);

                let srclang = match &subtitle_file.language {
                    Some(code) => format!(r#"srclang="{code}""#),
                    None => String::new()
                };

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


        let selector_options = video.subtitles
            .iter()
            .map(|subtitle_file| {
                let label = html_escape_outside_attribute(&subtitle_file.label);

                formatdoc!(r#"
                    <button>
                        <span class="active_marker_container">{active_marker_icon}</span> {label}
                    </button>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let subtitles_icon = icons::subtitles(Some(translations.subtitles));
        let t_no_subtitles = translations.no_subtitles;
        subtitle_controls = formatdoc!(r#"
            <div class="selector_control subtitles">
                <button class="cycle_subtitles dimmed" tabindex="-1">{subtitles_icon}</button>
                <div class="selector_options right_aligned">
                    <button class="active">
                        <span class="active_marker_container">{active_marker_icon}</span> {t_no_subtitles}
                    </button>
                    {selector_options}
                </div>
            </div>
        "#);
    } else {
        subtitle_controls = String::new();
        subtitle_tracks = String::new();
    };

    let mut version_selector_options = video.video_files_desc_by_fidelity()
        .iter()
        .map(|video_file| {
            let video_meta = video_file.video_meta_unchecked();

            let file_name_escaped = urlencoding::encode(&video_file.file_name);
            let height = video_meta.height;

            let size_humanized = human_size(video_file.file_meta.size);

            formatdoc!(r#"
                <button data-label="{height}p" data-src="{video_prefix}{file_name_escaped}">
                    <span class="active_marker_container">{active_marker_icon}</span> <strong>{height}p</strong>&nbsp;&nbsp;<small>{size_humanized}</small>
                </button>
            "#)
        })
        .collect::<Vec<String>>();

    let recommended_audio_file = video.recommended_audio_file();

    if let Some(audio_file) = recommended_audio_file {
        let file_name_escaped = urlencoding::encode(&audio_file.file_name);
        let size_humanized = human_size(audio_file.file_meta.size);

        let t_audio = translations.audio;
        let audio_option = formatdoc!(r#"
            <button data-label="{t_audio}" data-src="{video_prefix}{file_name_escaped}">
                <span class="active_marker_container">{active_marker_icon}</span> <strong>{t_audio}</strong>&nbsp;&nbsp;<small>{size_humanized}</small>
            </button>
        "#);

        version_selector_options.push(audio_option);
    }

    let version_selector_options = version_selector_options.join("\n");

    let default_version_label = recommended_video_file
        .map(|video_file| video_file.vertical_resolution_label())
        .unwrap_or_else(|| translations.audio.to_string());

    let version_controls = formatdoc!(r#"
        <div class="selector_control version" style="position: relative;">
            <button id="cycle_versions" tabindex="-1">{default_version_label}</button>
            <div class="selector_options centered">
                {version_selector_options}
            </div>
        </div>
    "#);

    let versions_ring_rendered = versions_ring(
        recommended_audio_file,
        recommended_video_file,
        translations,
        video,
        video_prefix
    );

    let player_outro = if let Some(playlist) = playlist {
        let index_suffix = site.index_suffix();
        let index_suffix_file_only = site.index_suffix_file_only();

        let playlist_label = playlist.title_or_slug_or_generic_label(translations);
        let playlist_label_escaped = html_escape_outside_attribute(&playlist_label);
        let videos = playlist.public_videos_in_playlist_order(language);

        let items = videos
            .iter()
            .enumerate()
            .map(|(index, video_iterated)| {
                let label = video_iterated.title_or_slug_or_generic_label(translations);
                let slug = video_iterated.slug_unchecked();

                let href_suffix = if embed {
                    format!("embed{index_suffix}")
                } else {
                    index_suffix_file_only.to_string()
                };

                let current;
                let href;
                let id_replay;
                let number_context;
                let replay_indicator;
                let video_iterated_prefix;
                if video_iterated.site_path == video.site_path {
                    current = "current";
                    href = format!("{video_prefix}{href_suffix}");
                    id_replay = r#"id="replay""#;

                    let count = videos.len();
                    let number = index + 1;
                    let t_current_video_xxx_of_xxx_in_xxx = translations.current_video_xxx_of_xxx_in_xxx(
                        number,
                        &playlist_label_escaped,
                        count
                    );

                    number_context = format!(r#"<span class="number_context">{t_current_video_xxx_of_xxx_in_xxx}</span>"#);
                    replay_indicator = icons::replay(translations.replay);
                    video_iterated_prefix = video_prefix.to_string();
                } else {
                    current = "";
                    href = format!("../{video_prefix}{slug}/{href_suffix}");
                    id_replay = "";
                    number_context = String::new();
                    replay_indicator = String::new();
                    video_iterated_prefix = format!("{video_prefix}../{slug}/");
                }

                let thumbnail_rendered = thumbnail(translations, video_iterated, &video_iterated_prefix, false);

                let mut details = Vec::new();

                if let Some(release_date) = video.release_date {
                    let date = localized_date_d_mmm_yyyy(release_date, site.language.chrono_locale);
                    details.push(date);
                }

                if let Some(video_meta) = &video_iterated
                    .highest_quality_video_file()
                    .and_then(|video_file| video_file.video_meta.as_ref()) {
                    let duration = casual_duration(video_meta.duration, translations);
                    details.push(duration);
                }

                let details = details.join(", ");

                formatdoc!(r#"
                    <a class="{current}" href="{href}" {id_replay}>
                        <div class="replay_indicator">{replay_indicator}</div>
                        <div class="description">
                            {number_context}
                            <span class="label">{label}</span>
                            <span class="details">
                                {details}
                            </span>
                        </div>
                        {thumbnail_rendered}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        formatdoc!(r#"
            <div class="outro_playlist_context">
                <h2 class="title">
                    <a href="..{index_suffix}">{playlist_label_escaped}</a>
                </h2>
                {items}
            </div>
        "#)
    } else {
        let replay_icon = icons::replay(translations.replay);
        let t_replay = translations.replay;
        formatdoc!(r#"
            <div class="outro_video_context">
                <button id="replay">
                    {replay_icon} {t_replay}
                </a>
            </div>
        "#)
    };

    let maximize_icon = icons::maximize(translations.maximize);
    let volume_loud_icon = icons::volume_loud(translations.loud_volume);

    let video_title = video.title_or_slug_or_generic_label(translations);
    let video_title_escaped = html_escape_outside_attribute(video_title);

    let embed_intro_extensions = if embed {
        let link_icon = icons::link(Some(translations.external_link));
        let site_url = site.absolute_url_unchecked();
        let video_url = match site.clean_urls {
            true => video_prefix.to_string(),
            false => format!("{video_prefix}index.html")
        };

        let intro_subtitle_controls = if video.subtitles.is_empty() {
            String::new()
        } else {
            let selector_options = video.subtitles
                .iter()
                .map(|subtitle_file| {
                    let label = html_escape_outside_attribute(&subtitle_file.label);

                    formatdoc!(r#"
                        <button>
                            <span class="active_marker_container">{active_marker_icon}</span> {label}
                        </button>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            let subtitles_icon = icons::subtitles(Some(translations.subtitles));
            let t_no_subtitles = translations.no_subtitles;

            formatdoc!(r#"
                <div class="selector_control subtitles">
                    <button class="cycle_subtitles dimmed icon_button" tabindex="-1">{subtitles_icon}</button>
                    <div class="selector_options right_aligned">
                        <button class="active">
                            <span class="active_marker_container">{active_marker_icon}</span> {t_no_subtitles}
                        </button>
                        {selector_options}
                    </div>
                </div>
            "#)
        };

        formatdoc!(r#"
            <a class="intro_heading" href="{video_url}" target="_blank">
                {link_icon} <span class="label">{video_title_escaped} – {site_url}</span>
            </a>
            <div class="intro_controls_wrapper">
                <div class="intro_controls">
                    <button class="icon_button cycle_volume">{volume_loud_icon}</button>
                    {intro_subtitle_controls}
                    <button class="icon_button toggle_fullscreen">{maximize_icon}</button>
                </div>
            </div>
        "#)
    } else {
        String::new()
    };

    let aspect_ratio;
    let duration;
    let src_escaped;
    if let Some(video_file) = recommended_video_file {
        let video_file_meta = video_file.video_meta.as_ref().unwrap();

        aspect_ratio = video_file_meta.aspect_ratio();
        duration = video_file_meta.duration;
        src_escaped = urlencoding::encode(&video_file.file_name);
    } else {
        let audio_file = recommended_audio_file.unwrap();
        let audio_file_meta = audio_file.audio_meta.as_ref().unwrap();

        aspect_ratio = site.poster_aspect.numeric();
        duration = audio_file_meta.duration;
        src_escaped = urlencoding::encode(&audio_file.file_name);
    }

    let duration_formatted = precise_duration(duration);

    let play_icon = icons::play(translations.play);
    formatdoc!(r#"
        <style>
            :root {{ --video-aspect: {aspect_ratio}; }}
        </style>
        <div class="player_wrapper" data-state="intro">
            <div class="player_intro">
                {versions_ring_rendered}
                {embed_intro_extensions}
            </div>
            <div class="player_outro">
                {player_outro}
            </div>
            <div class="player_controls">
                <div class="timeline">
                    <div class="bar backdrop"></div>
                    <div class="bar progress"></div>
                    <input aria-valuenow="0" aria-valuetext="00:00" max="{duration}" min="0" step="any" type="range" value="0">
                    <div class="bar outline"></div>
                </div>
                <div class="control_pane">
                    <div class="left_playback_controls">
                        <button id="toggle_playback">{play_icon}</button>
                        <button class="cycle_volume">{volume_loud_icon}</button>
                        <button id="speed"><span class="multiplier">1.0</span><span class="x">x</span></button>
                        <div>
                            <span id="time_current">00:00</span> / <span id="time_total">{duration_formatted}</span>
                        </div>
                    </div>
                    <div class="right_playback_controls">
                        {version_controls}
                        {subtitle_controls}
                        <button class="toggle_fullscreen">{maximize_icon}</button>
                    </div>
                </div>
            </div>
            <video data-duration="{duration}" playsinline {poster} preload="none" src="{video_prefix}{src_escaped}">
                {subtitle_tracks}
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
    let replay_icon = icons::replay(translations.replay);
    let subtitles_icon = icons::subtitles(Some(translations.subtitles));
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
        <template id="replay_icon">
            {replay_icon}
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

/// A ring with slices and a play button in the middle, displayed on
/// the "intro" screen of the video player. Lets people choose the resolution
/// (or audio-only version) to watch the video in (and simultaneously start
/// playback).
fn versions_ring(
    recommended_audio_file: Option<&AudioFile>,
    recommended_video_file: Option<&VideoFile>,
    translations: &Translations,
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

    if let Some(audio_file) = recommended_audio_file {
        let file_name = &audio_file.file_name;
        let label = translations.audio.to_string();
        let size = audio_file.file_meta.size;

        sizes.push((file_name, label, size));
        total_size += size;

        if size > largest_size {
            largest_size = size;
        }
    }

    let video_files = video.video_files_asc_by_fidelity();

    if !video_files.is_empty() {
        for video_file in &video_files {
            let file_name = &video_file.file_name;
            let label = video_file.vertical_resolution_label();
            let size = video_file.file_meta.size;

            sizes.push((file_name, label, size));
            total_size += size;

            if size > largest_size {
                largest_size = size;
            }
        }
    }

    // Draws a ring segment - we start at 12 o'clock and go clockwise.
    let slice = |
        arc_angle: f32,
        begin_angle: f32,
        default: bool,
        label: &str,
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
        let text_label_y = RADIUS + -text_angle_rad.cos() * TEXT_RADIUS;
        let text_size_y = RADIUS + -text_angle_rad.cos() * TEXT_RADIUS + TEXT_SPACING;

        let play_button = if default {
            // The perceived visual center of the triangle lies slightly
            // off-center, this factor is used to shift it a bit to make it
            // look perfectly centered.
            const PLAY_BUTTON_X_CENTER_OFFSET: f32 = 0.59;
            const PLAY_BUTTON_RATIO: f32 = 1.197655;
            const PLAY_BUTTON_WIDTH: f32 = RADIUS * 0.2;
            const PLAY_BUTTON_HEIGHT: f32 = PLAY_BUTTON_RATIO * PLAY_BUTTON_WIDTH;
            let a_x = RADIUS - PLAY_BUTTON_WIDTH * (1.0 - PLAY_BUTTON_X_CENTER_OFFSET);
            let a_y = RADIUS - PLAY_BUTTON_HEIGHT * 0.5;
            let b_x = RADIUS + PLAY_BUTTON_WIDTH * PLAY_BUTTON_X_CENTER_OFFSET;
            let b_y = RADIUS;
            let c_x = RADIUS - PLAY_BUTTON_WIDTH * (1.0 - PLAY_BUTTON_X_CENTER_OFFSET);
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
                data-label="{label}"
                data-src="{src}"
                tabindex="0">
                {play_button}
                {slice}
                <rect class="backdrop" filter="url(#text_shadow)" rx="5"/>
                <text text-anchor="middle" x="{text_x}" y="{text_label_y}">
                    {label}
                </text>
                <text class="size" text-anchor="middle" x="{text_x}" y="{text_size_y}">
                    {size}
                </text>
                <title>{label} {size}</title>
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

        let default =
            recommended_video_file.is_some_and(|video_file| video_file.file_name == **file_name) ||
            recommended_audio_file.is_some_and(|audio_file| audio_file.file_name == **file_name);
        let file_name_escaped = urlencoding::encode(file_name);
        let src = format!("{video_prefix}{file_name_escaped}");
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

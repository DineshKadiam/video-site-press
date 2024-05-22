use std::path::Path;
use std::sync::Arc;

use indoc::formatdoc;

use crate::{Language, Link, PosterFile, Site, ThemeKind, Translations, Video, VideoFile};
use crate::build::outline::Smartcrumb;
use crate::icons;
use crate::util::{casual_duration, human_size, precise_duration};

pub fn copy_link(site: &Site, path: &str) -> String {
    let data_url = match &site.base_url {
        Some(base_url) => {
            let url = base_url.value.join(path).unwrap().to_string();
            format!(r#"data-url="{url}""#)
        }
        None => String::new()
    };

    let t_copied = site.language.translations.copied;
    let t_copy_link = site.language.translations.copy_link;
    let t_failed = site.language.translations.failed;

    let copy_link_icon = icons::copy_link(t_copy_link);
    let failure_icon = icons::failure(t_failed);
    let success_icon = icons::success(t_copied);
    format!(r#"
        <div>
            <button {data_url} id="copy-link">
                <span class="icon">{copy_link_icon}</span> {t_copy_link}
            </button>
            <template id="copy_link_icon">
                {copy_link_icon}
            </template>
            <template id="failure_icon">
                {failure_icon}
            </template>
            <template id="success_icon">
                {success_icon}
            </template>
        </div>
    "#)
}


pub fn downloads(site: &Site, video: &Video) -> String {
    if video.versions.is_empty() {
        return String::new();
    }

    let mut versions_with_meta: Vec<&VideoFile> = video.versions
        .iter()
        .filter(|version| version.video_meta.is_some())
        .collect();

    versions_with_meta.sort_unstable_by(|a, b| {
        a.video_meta.as_ref().unwrap().height.cmp(
            &b.video_meta.as_ref().unwrap().height
        )
    });

    let format_buttons = versions_with_meta
        .iter()
        .map(|version| {
            // TODO: Clean up/better/more complete concept for how to build file names (e.g. maybe hide original names altogether!)
            let file_name = version.file_name.as_str();
            let video_meta = &version.video_meta.as_ref().unwrap();

            let height = video_meta.height;
            let width = video_meta.width;

            let slug = video.slug_opt().unwrap_or_else(|| "video");
            let download_name = Path::new(file_name).with_file_name(format!("{slug}_{width}x{height}")).to_string_lossy().to_string();

            let format_label = format!("{width}×{height}");
            let vertical_resolution_label = format!("{height}p");

            let size = human_size(version.file_meta.size);

            let t_no_audio = site.language.translations.no_audio;
            let audio_codec = video_meta.audio_codec.as_deref().unwrap_or(t_no_audio);

            let t_no_video = site.language.translations.no_video;
            let video_codec = video_meta.video_codec.as_deref().unwrap_or(t_no_video);

            let extension = Path::new(&version.file_name)
                .extension()
                .unwrap_or(std::ffi::OsStr::new(""))
                .to_string_lossy();

            formatdoc!(r#"
                <a download="{download_name}" class="align apart flex" href="{file_name}">
                    <div>
                        <div class="large_font">{vertical_resolution_label}</div>
                        <small>{format_label}, {audio_codec}, {video_codec}, {extension}</small>
                    </div>
                    <div class="large_font">{size}</div>
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let t_subtitles = site.language.translations.subtitles;
    let subtitle_buttons = video.subtitles
        .iter()
        .map(|subtitle_file| {
            let file_name = &subtitle_file.file_name;
            let language = match &subtitle_file.language {
                Some(language_code) => format!(r#"<div class="large_font">{language_code}</div>"#),
                None => String::new()
            };

            formatdoc!(r#"
                <a download="{file_name}" class="align apart flex" href="{file_name}">
                    <div>
                        <div class="large_font">{file_name}</div>
                        <small>{t_subtitles}</small>
                    </div>
                    {language}
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let t_download = site.language.translations.download;
    let download_icon = icons::download(t_download);
    formatdoc!(r#"
        <button id="download">
            {download_icon} {t_download}
        </button>
        <dialog id="download_dialog">
            {format_buttons}
            {subtitle_buttons}
            <form method="dialog">
                <button>Close</button>
            </form>
        </dialog>
    "#)
}

pub fn layout(
    root_prefix: &str,
    site: &Site,
    body: &str,
    title: &str,
    outline: &[Smartcrumb]
) -> String {
    // TODO: Hash assets (cache expiration)
    let head = formatdoc!(r#"
        <head>
            <title>{title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <script defer src="{root_prefix}scripts.js"></script>
            <link rel="stylesheet" href="{root_prefix}styles.css">
        </head>
    "#);

    let outline_rendered = outline
        .iter()
        .enumerate()
        .map(|(index, smartcrumb)| {
            let href = "../".repeat(outline.len() - 1 - index);
            smartcrumb.breadcrumb(&href, site)
        })
        .collect::<Vec<String>>()
        .join(" › ");

    let dark_icon = icons::dark(site.language.translations.dark_color_scheme);
    let light_icon = icons::light(site.language.translations.light_color_scheme);
    let theme_toggle = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => formatdoc!(r#"
            <div id="theme-toggles">
                <button class="theme_toggle" id="toggle-dark">{dark_icon}</button>
                <button class="theme_toggle" id="toggle-light">{light_icon}</button>
            </div>
        "#),
        ThemeKind::Static(_) => String::new()
    };

    let hyper8_version = env!("CARGO_PKG_VERSION");

    let with_sidebar = if outline.len() > 1 { "with_sidebar" } else { "" };

    let logo_icon = icons::logo(site.language.translations.hyper8_logo);
    let body = formatdoc!(r##"
        <body>
            <div class="layout {with_sidebar}">
                <div class="growing">
                    <div class="center">
                        <header>
                            {outline_rendered}
                        </header>
                        {body}
                    </div>
                </div>
                <footer>
                    <a class="system_info" href="https://simonrepp.com/hyper8" target="_blank" title="Hyper 8 Video System {hyper8_version}">
                        {logo_icon} Hyper 8
                    </a>
                    {theme_toggle}
                </footer>
            </div>
        </body>
    "##);

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
    links
        .iter()
        .map(|link| {
            let label = link.label.as_ref().map(|label| label.clone()).unwrap_or_else(|| link.url.to_string());
            let rel_me = if link.rel_me { r#"rel="me""# } else { "" };
            let style_hidden = if link.hidden { r#"style="display: none;""# } else { "" };
            let url = &link.url;

            formatdoc!(r#"
                <div>
                    <a class="link" href="{url}" {rel_me} {style_hidden}>{external_icon} <span>{label}</span></a>
                </div>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn player(
    site: &Site,
    video: &Video,
    next_video: Option<Arc<Video>>
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

                format!(r#"<img {alt}class="thumbnail" src="poster.jpg">"#)
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
                format!(r#"poster="poster_{width}x{height}.jpg""#)
            }
            None => format!(r#"poster="poster.jpg""#)
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
                    src="{index}.vtt"
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

    let versions_ring_rendered = versions_ring(recommended_version, video);

    let t_next_video = site.language.translations.next_video;
    let next_video_button = match next_video {
        Some(video) => format!(r#"<a href="../{}{index_suffix}">{t_next_video}</a>"#, video.slug()),
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
            <video data-duration="{duration}" {poster} preload="none" src="{src}">
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

pub fn unlisted_badge(unlisted: bool, translations: &Translations) -> String {
    if unlisted {
        let t_not_public = translations.not_public;
        format!(r#"<span class="unlisted_badge">{t_not_public}</span>"#)
    } else {
        String::new()
    }
}

fn versions_ring(recommended_version: &VideoFile, video: &Video) -> String {
    let radius = 190.0;
    let diameter = radius * 2.0;

    let mut sizes = Vec::new();
    let mut total_size: u64 = 0;

    let mut largest_size = 0;

    for version in &video.versions {
        let src = &version.file_name;
        let size = version.file_meta.size;
        let resolution = version.vertical_resolution_label();
        sizes.push((size, src, resolution));
        total_size += size;

        if size > largest_size {
            largest_size = size;
        }
    }

    sizes.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    // Draws a ring segment. In clock terms we start at 12 o'clock and we go clockwise.
    let slice = |
        arc_angle: f32,
        begin_angle: f32,
        default: bool,
        src: &str,
        tab_index: usize,
        resolution: &str,
        size: &str
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
                    tabindex="{tab_index}">
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

    for (index, (size, src, title)) in sizes.iter().enumerate() {
        let arc_angle = if sizes.len() == 1 {
            usable_angle
        } else {
            (*size as f32 / total_size as f32) * usable_angle
        };

        let size_humanized = human_size(*size);

        let default = &recommended_version.file_name == *src;
        let slice_rendered = slice(
            arc_angle,
            begin_angle,
            default,
            src,
            index + 1,
            title,
            &size_humanized
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

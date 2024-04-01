use std::path::Path;

use indoc::formatdoc;

use crate::{Site, ThemeKind, Video, VideoFile};
use crate::build::Breadcrumb;
use crate::util::human_size;

pub struct VideoRepresentation {
    pub saving_factor: f32,
    pub saving_size: u64,
    pub version: VideoFile
}

const ICON_DARK: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m31.934 8.834c-3.9179 0.005803-7.8855 1.0068-11.518 3.1035-11.069 6.3911-14.869 20.575-8.4785 31.645 6.3911 11.069 20.575 14.871 31.645 8.4805h0.001953c11.069-6.3911 14.869-20.577 8.4785-31.646-3.9944-6.9178-11.033-10.997-18.453-11.523-0.55651-0.039474-1.1161-0.059423-1.6758-0.058594zm0.013672 2.1836c0.5065-0.001032 1.0121 0.017281 1.5156 0.052734 6.7142 0.47271 13.077 4.1669 16.701 10.443 5.7976 10.043 2.3646 22.852-7.6777 28.65-0.81094 0.46815-1.6406 0.86959-2.4824 1.2188-4.275-1.0509-10.322-6.9759-15.115-15.277-4.7914-8.2993-6.899-16.498-5.6738-20.727 0.72372-0.55489 1.4874-1.0745 2.2988-1.543 3.2953-1.9023 6.8881-2.8111 10.434-2.8184z"/>
</svg>
"#;

const ICON_LIGHT: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m29.68 3.6836v9.2344h4.6406v-9.2344h-4.6406zm-16.199 6.5156-3.2812 3.2832 6.5996 6.5957 3.2812-3.2812-6.5996-6.5977zm37.039 0-6.5996 6.5977 3.2812 3.2812 6.5996-6.5957-3.2812-3.2832zm-18.52 10.086a11.715 11.715 0 0 0-11.715 11.715 11.715 11.715 0 0 0 11.715 11.713 11.715 11.715 0 0 0 11.715-11.713 11.715 11.715 0 0 0-11.715-11.715zm-28.316 9.3926v4.6426h9.2324v-4.6426h-9.2324zm47.4 0v4.6426h9.2324v-4.6426h-9.2324zm-34.285 14.242-6.5996 6.5957 3.2812 3.2832 6.5996-6.5977-3.2812-3.2812zm30.402 0-3.2812 3.2812 6.5996 6.5977 3.2812-3.2832-6.5996-6.5957zm-17.521 7.1602v9.2344h4.6406v-9.2344h-4.6406z"/>
</svg>
"#;

pub fn layout(
    root_prefix: &str,
    site: &Site,
    body: &str,
    title: &str,
    breadcrumbs: &[Breadcrumb]
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

    let breadcrumbs_rendered = breadcrumbs
        .iter()
        .enumerate()
        .map(|(index, breadcrumb)| {
            let href = "../".repeat(breadcrumbs.len() - 1 - index);
            let label = breadcrumb.label();
            format!(r#"<a href="{href}">{label}</a>"#)
        })
        .collect::<Vec<String>>()
        .join(" › ");

    let theme_toggle = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => formatdoc!(r#"
            <div id="theme-toggles">
                <button class="theme_toggle" id="toggle-dark">{ICON_DARK}</button>
                <button class="theme_toggle" id="toggle-light">{ICON_LIGHT}</button>
            </div>
        "#),
        ThemeKind::Static(_) => String::new()
    };

    let hyper8_version = env!("CARGO_PKG_VERSION");

    let body = formatdoc!(r##"
        <body>
            <div class="center">
                <header>
                    {breadcrumbs_rendered} {theme_toggle}
                </header>
                {body}
                <footer style="margin-top: 3rem">
                    <a class="system_info" href="https://simonrepp.com/hyper8" target="_blank">Hyper 8 Video System {hyper8_version}</a> 
                </footer>
            </div>
        </body>
    "##);

    let lang_code = &site.language.code;
    let dir_attribute = if site.language.rtl { r#"dir="rtl""# } else { "" };

    formatdoc!(r#"
        <!doctype html>
        <html {dir_attribute} lang="{lang_code}">
            {head}
            {body}
        </html>
    "#)
}

fn compute_representations(versions: &[VideoFile]) -> Vec<VideoRepresentation> {
    let mut max_size = 0;

    for version in versions {
        if version.file_meta.size > max_size { max_size = version.file_meta.size; }
    }

    let mut representations = versions
        .iter()
        .map(|version| {
            let saving_size = max_size - version.file_meta.size;
            let saving_factor = 1.0 - version.file_meta.size as f32 / max_size as f32;

            VideoRepresentation {
                saving_factor,
                saving_size,
                version: version.clone()
            }
        })
        .collect::<Vec<VideoRepresentation>>();

    representations.sort_by(|a, b| b.saving_size.cmp(&a.saving_size));

    representations
}

pub fn player(video: &Video, video_prefix: &str) -> String {
    if !video.versions.is_empty() {
        let downloads = video.versions
            .iter()
            .map(|version| {
                // TODO: Clean up/better/more complete concept for how to build file names (e.g. maybe hide original names altogether!)
                let file_name = version.file_name.as_str();
                let (format_label, download_name) = if let Some(video_meta) = &version.video_meta {
                    let height = video_meta.height;
                    let width = video_meta.width;

                    let slug = video.slug_opt().map(|slug| format!("{slug}_")).unwrap_or_else(|| format!("video_"));
                    let download_name = Path::new(file_name).with_file_name(format!("{slug}_{width}x{height}")).to_string_lossy().to_string();

                    (
                        format!("{width}×{height}"),
                        download_name
                    )
                } else {
                    (
                        String::new(),
                        file_name.to_owned()
                    )
                };
                let size = human_size(version.file_meta.size);

                formatdoc!(r#"
                    <a download="{download_name}" href="{video_prefix}{file_name}">
                        <strong>Download {format_label}</strong> {size}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("<br>");

        let representations = compute_representations(&video.versions)
            .iter()
            .map(|representation| {
                let filename = &representation.version.file_name;
                let format_label = "TODO";
                let size = human_size(representation.version.file_meta.size);
                let saving_factor = representation.saving_factor;
                let saving_percent = (saving_factor * 100.0) as u8;

                formatdoc!(r#"
                    <div class="chart" onclick="document.querySelector('video').setAttribute('src', '{video_prefix}{filename}'); document.querySelector('video').play();">
                        <span class="size">{size}</span>
                        <span class="label">{format_label}</span>
                        <div class="emissions" style="width: calc((1 - {saving_factor}) * 33rem);">
                            <div class="bar"></div>
                        </div>
                        <div class="savings" style="width: calc({saving_factor} * 33rem);">
                            <div class="bar"></div>
                        </div>
                        <span class="savings_percent" title="-{saving_percent}% less emissions">-{saving_percent}%</span>
                    </div>
                "#)
            })
            .collect::<Vec<String>>()
            .join("");

        let height = video.versions.first().unwrap().video_meta.as_ref().unwrap().height as f32;
        let width = video.versions.first().unwrap().video_meta.as_ref().unwrap().width as f32;
        let scale = if width > height { 16.0 / width } else { 9.0 / height };

        let experiment = compute_representations(&video.versions)
            .iter()
            .rev()
            .map(|representation| {
                let filename = &representation.version.file_name;
                // let format_label = representation.version.format.label();
                // let size = human_size(representation.version.size);
                let saving_factor = representation.saving_factor;
                // let saving_percent = (saving_factor * 100.0) as u8;
                let divider = 1.0 - saving_factor; // e.g. 1/4
                let sqrt = divider.sqrt() * 100.0;
                // let relative_size_factor = (1.0 - saving_factor.exp2()) * 100.0;
                
                // 100MB = 1           16x16
                // 25MB = 1/4 = 0.25   8x8  => *2
                // 6.25MB = 1/16 = 0.0625   4z4   => *4

                formatdoc!(r#"
                    <div onclick="document.querySelector('video').setAttribute('src', '{video_prefix}{filename}'); document.querySelector('video').play();"
                        style="
                            border: 1px solid black;
                            left: 0;
                            bottom: 0;
                            height: {sqrt}%;
                            position: absolute;
                            width: {sqrt}%;
                        ">
                    </div>
                "#)
            })
            .collect::<Vec<String>>()
            .join("");

        let smallest_version = video.versions.first().unwrap();

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

        let src = format!("{}{}", video_prefix, smallest_version.file_name);

        let experiment_combined = formatdoc!(
            r#"
                <div style="height: {height}rem; position: relative; width: {width}rem;">{experiment}</div>
            "#,
            width = width * scale,
            height = height * scale
        );

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

        formatdoc!(r#"
            <video controls {poster} preload="none" src="{src}">
                {subtitles}
            </video>
            <br><br>
            <div>{representations}</div>
            <br>
            {experiment_combined}
            <br>
            <div>{downloads}</div>
        "#)
    } else {
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

        formatdoc!(r#"
            <div style="border: 1px solid red; height: 12.8rem; width: 19.2rem;">
                {poster}
                At the time of publishing the video was not yet rendered for viewing.
            </div>
        "#)
    }
}

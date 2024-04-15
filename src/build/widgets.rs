use std::path::Path;

use indoc::formatdoc;

use crate::{Site, Video, VideoFile};
use crate::icons::COPY_LINK_SVG;
use crate::util::{casual_duration, human_size};

pub struct VideoRepresentation {
    pub saving_factor: f32,
    pub saving_size: u64,
    pub version: VideoFile
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

pub fn copy_link(site: &Site, path: &str) -> String {
    let data_url = match &site.base_url {
        Some(base_url) => {
            let url = base_url.value.join(path).unwrap().to_string();
            format!(r#"data-url="{url}""#)
        }
        None => String::new()
    };

    let t_copy_link = &site.language.translations.copy_link;

    format!(r#"
        <div>
            <button {data_url} id="copy-link"><span class="icon">{COPY_LINK_SVG}</span> {t_copy_link}</button>
        </div>
    "#)
}


pub fn downloads(video: &Video) -> String {
    video.versions
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
                <a download="{download_name}" href="{file_name}">
                    Download {format_label} <small>{size}</small>
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("<br>")
}

pub fn player(site: &Site, video: &Video, video_prefix: &str) -> String {
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

        let t_video_not_yet_available = &site.language.translations.video_not_yet_available;

        return formatdoc!(r#"
            <div style="border: 1px solid red; height: 12.8rem; width: 19.2rem;">
                {poster}
                {t_video_not_yet_available}
            </div>
        "#);
    }

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
        <!--
            {representations}
            {experiment_combined}
        -->
    "#)
}

pub fn thumbnail(video: &Video, video_prefix: &str, with_duration: bool) -> String {
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

            let duration = match &video.highest_resolution_version().and_then(|version| version.video_meta.as_ref()) {
                Some(video_meta) => {
                    let duration = casual_duration(video_meta.duration);
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

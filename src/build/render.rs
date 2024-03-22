use indoc::formatdoc;

use crate::{Site, Video, VideoFile};
use crate::build::Breadcrumb;
use crate::util::{casual_duration, human_size};


pub struct VideoRepresentation {
    pub saving_factor: f32,
    pub saving_size: u64,
    pub version: VideoFile
}

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
    
    let body = formatdoc!(r#"
        <body>
            <div class="center">
                <header>
                    {breadcrumbs_rendered}
                </header>
                {body}
                <footer>
                    <span class="logo">Hyper 8</span>
                </footer>
            </div>
        </body>
    "#);

    formatdoc!("
        <!doctype html>
        <html>
            {head}
            {body}
        </html>
    ")
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
                let filename = &version.file_name;
                // let format_label = &version.format.label();
                let format_label = "[TODO: format_label]";
                let size = human_size(version.file_meta.size);

                formatdoc!(r#"
                    <a download="{format_label}.mp4" href="{video_prefix}{filename}">
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

        formatdoc!(r#"
            <video controls {poster}preload="none" src="{src}">
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

// /// Renders the index page of a playlist (lists videos only)
// pub fn playlist(
//     build_dir: &Path,
//     playlist: &Playlist,
//     site: &Site,
//     videos: &[Arc<Video>]
// ) {
//     let list = videos
//         .iter()
//         .map(|video| {
//             let slug = video.slug();
//             let title = video.title.as_deref().unwrap_or(video.slug());
//             let video_prefix = format!("../../{slug}/");
//             let video_thumbnail = thumbnail(video, &video_prefix);

//             formatdoc!(r#"
//                 <li>
//                     <a href="{slug}/">
//                         {video_thumbnail}
//                         <div>
//                             {title}
//                         </div>
//                     </a>
//                 </li>
//             "#)
//         })
//         .collect::<Vec<String>>()
//         .join("\n");

//     let description = if let Some(description) = &playlist.description { description } else { "" };
//     let title = playlist.title.as_deref().unwrap_or(playlist.slug());

//     let body = formatdoc!(r#"
//         <div>
//             <h1>{title}</h1>
//             <p>
//                 {description}
//             </p>
//             <div>
//                 <a href="TODO">Share</a>
//             </div>
//             <ol>
//                 {list}
//             </ol>
//         </div>
//     "#);

//     let html = layout("../../", site, &body, title);

//     let playlist_dir = build_dir.join("playlists").join(playlist.slug());
//     fs::create_dir_all(&playlist_dir).unwrap();

//     let index_path = playlist_dir.join("index.html");
//     fs::write(index_path, html).unwrap();
// }

// /// Renders the page for a video in the context of a playlist
// pub fn playlist_video(
//     build_dir: &Path,
//     playlist: &Playlist,
//     site: &Site,
//     videos: &[Arc<Video>],
//     video: &Video
// ) {
//     let root_prefix = "../../../";
//     let video_prefix = format!("{root_prefix}{}/", video.slug());
//     let description = if let Some(description) = &video.description { description } else { "" };
//     let title = video.title.as_deref().unwrap_or(video.slug());

//     let player_rendered = player(video, &video_prefix);

//     let videos_rendered = videos
//         .iter()
//         .enumerate()
//         .map(|(index, iterated_video)| {
//             let active = iterated_video.slug() == video.slug();
//             let class = if active { r#" class="active""# } else { "" };
//             let number = index + 1;
//             // TODO: Use poster assets (different sizes)
//             let poster = match iterated_video.poster {
//                 Some(_) => {
//                     let iterated_video_prefix = format!("{root_prefix}{}/", iterated_video.slug());
//                     format!(r#"<img {alt}class="thumbnail" src="{iterated_video_prefix}poster.jpg">"#)
//                 }
//                 None => String::new()
//             };
//             let slug = iterated_video.slug();
//             let title = iterated_video.title.as_deref().unwrap_or(video.slug());

//             formatdoc!(r#"
//                 <a {class}href="../{slug}/">
//                     {poster}
//                     {number} {title}
//                 </a>
//             "#)
//         })
//         .collect::<Vec<String>>()
//         .join("\n");

//     let body = formatdoc!(r#"
//         <div>
//             <h1>{title}</h1>
//             <div class="split">
//                 <div>
//                     {player_rendered}
//                     <p>
//                         {description}
//                     </p>
//                     <div>
//                         <a href="TODO">Share</a>
//                     </div>
//                 </div>
//                 <div class="sidebar">
//                     {videos_rendered}
//                 </div>
//             </div>
//         </div>
//     "#);

//     let html = layout(root_prefix, site, &body, playlist.title.as_deref().unwrap_or(playlist.slug()));

//     let playlist_video_dir = build_dir
//         .join("playlists")
//         .join(playlist.slug())
//         .join(video.slug());

//     fs::create_dir_all(&playlist_video_dir).unwrap();

//     let index_path = playlist_video_dir.join("index.html");
//     fs::write(index_path, html).unwrap();
// }

pub fn thumbnail(video: &Video, video_prefix: &str) -> String {
    let duration = match &video.versions.first().and_then(|version| version.video_meta.as_ref()) {
        Some(video_meta) => {
            let duration = casual_duration(video_meta.duration);
            formatdoc!(r#"<span class="duration">{duration}</span>"#)
        }
        None => String::new()
    };

    match &video.poster {
        Some(poster) => {
            let alt = if let Some(description) = &poster.description {
                format!(r#"alt="{description}" "#)
            } else {
                String::new()
            };

            if let Some(poster_assets) = &poster.assets {
                let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);
                let src = image_attributes.src;
                let srcset = image_attributes.srcset;

                formatdoc!(r#"
                    <div class="thumbnail_wrapper">
                        <img {alt}class="thumbnail" src="{src}" srcset="{srcset}">
                        {duration}
                    </div>
                "#)
            } else {
                formatdoc!(r#"
                    <div class="thumbnail_wrapper">
                        <img {alt}class="thumbnail" src="{video_prefix}poster.jpg">
                        {duration}
                    </div>
                "#)
            }
        },
        None => String::new()
    }
}

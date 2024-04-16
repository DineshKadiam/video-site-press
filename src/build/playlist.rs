use std::fs;

use indoc::formatdoc;

use crate::{Banner, Context, Playlist, Site};
use crate::build::Breadcrumb;
use crate::build::render::layout;
use crate::build::widgets::thumbnail;

pub fn banner_html(banner: &Banner, playlist_prefix: &str) -> String {
    let alt = if let Some(description) = &banner.description {
        format!(r#"alt="{description}" "#)
    } else {
        String::new()
    };

    if let Some(poster_assets) = &banner.assets {
        // TODO: Adjust up_to_xxx (depends on final content width we're using)
        let image_attributes = poster_assets.img_attributes_up_to_1280("banner", playlist_prefix);
        let src = image_attributes.src;
        let srcset = image_attributes.srcset;

        formatdoc!(r#"<img {alt}src="{src}" srcset="{srcset}">"#)
    } else {
        formatdoc!(r#"<img {alt}src="{playlist_prefix}banner.jpg">"#)
    }
}

pub fn playlist_html(
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    breadcrumbs: &Vec<Breadcrumb>
) {
    let mut body = String::new();
    let index_suffix = context.index_suffix();

    let title = if playlist.path == "" {
        playlist.title.as_deref().unwrap_or("Hyper 8")
    } else {
        playlist.title.as_deref().unwrap_or_else(|| playlist.slug())
    };

    if let Some(banner) = &playlist.banner {
        let playlist_prefix = "";
        let img = banner_html(banner, playlist_prefix);
        body.push_str(&img);
    }

    body.push_str(&format!("<h1>{title}</h1>\n\n"));

    if let Some(description) = &playlist.description {
        body.push_str(&format!("<p>{description}</p>\n\n"));
    }

    if !playlist.videos.is_empty() {
        let videos = playlist.videos
            .iter()
            .filter(|video| !video.unlisted)
            .enumerate()
            .map(|(index, video)| {
                let number = index + 1;
                let slug = video.slug();
                let label = video.label();
                let video_prefix = format!("{slug}/");
                let video_thumbnail = thumbnail(video, &video_prefix, true);

                formatdoc!(r#"
                    <a href="{slug}{index_suffix}">
                        <div>{video_thumbnail}</div>
                        <div class="description">
                            {number} {label}
                        </div>
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let videos_rendered = formatdoc!(r#"
            <h2>Videos</h2>

            <div class="playlist">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    let root_prefix = if playlist.path == "" { "".to_string() } else { "../".repeat(playlist.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, breadcrumbs);

    let playlist_dir = context.build_dir.join(&playlist.path);
    fs::create_dir_all(&playlist_dir).unwrap();
    let index_path = playlist_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
use std::fs;

use indoc::formatdoc;

use crate::{Banner, Context, Playlist, Site};
use crate::build::outline::Smartcrumb;
use crate::build::widgets::{layout, thumbnail, unlisted_badge};

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
    outline: &[Smartcrumb]
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

    let unlisted = unlisted_badge(playlist.unlisted, &site.language.translations);

    body.push_str(&format!("<h1>{title} {unlisted}</h1>\n\n"));

    if let Some(description) = &playlist.description {
        body.push_str(&format!("<p>{description}</p>\n\n"));
    }

    let videos = playlist.public_videos_asc_by_release_date();

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .enumerate()
            .map(|(index, video)| {
                let number = index + 1;
                let slug = video.slug();
                let label = video.label(&site.language.translations);
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

        let t_videos = &site.language.translations.videos;
        let videos_rendered = formatdoc!(r#"
            <h2>{t_videos}</h2>

            <div class="playlist">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    if !outline.is_empty() {
        let outline_items = outline
            .iter()
            .filter(|smartcrumb| !smartcrumb.is_playlist())
            .enumerate()
            .map(|(index, smartcrumb)| {
                let prefix = "../".repeat(outline.len() - 1 - index);
                smartcrumb.context_for_playlist(context, playlist, &prefix, site)
            })
            .collect::<Vec<String>>()
            .join("\n");

        body.push_str(&format!(r#"<div class="outline">{outline_items}</div>"#));
    }

    let root_prefix = if playlist.path == "" { "".to_string() } else { "../".repeat(playlist.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, outline);

    let playlist_dir = context.build_dir.join(&playlist.path);
    fs::create_dir_all(&playlist_dir).unwrap();
    let index_path = playlist_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
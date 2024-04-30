use std::fs;

use indoc::formatdoc;

use crate::{Collection, Context, Site};
use crate::build::Smartcrumb;
use crate::build::playlist::banner_html;
use crate::build::widgets::{layout, thumbnail};

pub fn collection_html(
    context: &Context,
    collection: &Collection,
    site: &Site,
    outline: &[Smartcrumb]
) {
    let mut body = String::new();
    let index_suffix = context.index_suffix();

    let title = if collection.path == "" {
        collection.title.as_deref().unwrap_or("Hyper 8")
    } else {
        collection.title.as_deref().unwrap_or_else(|| collection.slug())
    };

    body.push_str(&format!("<h1>{title}</h1>\n\n"));

    if let Some(description) = &collection.description {
        body.push_str(&format!("<p>{description}</p>\n\n"));
    }

    if !collection.videos.is_empty() {
        let videos = collection.videos
            .iter()
            .filter(|video| !video.unlisted)
            .map(|video| {
                let slug = &video.slug();
                let title = video.title.as_deref().unwrap_or_else(|| video.slug());
                let video_prefix = format!("{slug}/");

                let video_thumbnail = thumbnail(video, &video_prefix, true);

                formatdoc!(r#"
                    <a href="{slug}{index_suffix}">
                        {video_thumbnail}
                        <div>{title}</div>
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_videos = &site.language.translations.videos;
        let videos_rendered = formatdoc!(r#"
            <h2>{t_videos}</h2>

            <div class="videos_grid">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    if !collection.playlists.is_empty() {
        let playlists = collection.playlists
            .iter()
            .map(|playlist| {
                let slug = playlist.slug();
                let title = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());

                let playlist_prefix = format!("{slug}/");

                let banner = if let Some(banner) = &playlist.banner {
                    banner_html(banner, &playlist_prefix)
                } else {
                    String::new()
                };

                formatdoc!(r#"
                    {banner}
                    <a href="{slug}{index_suffix}">{title}</a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_playlists = &site.language.translations.playlists;
        let playlists_rendered = formatdoc!(r#"
            <h2>{t_playlists}</h2>

            <div>
                {playlists}
            </div>
        "#);

        body.push_str(&playlists_rendered);
    }

    for subcollection in &collection.subcollections {
        let subcollection_slug = subcollection.slug();
        let title = &subcollection.title.as_deref().unwrap_or_else(|| subcollection.slug());

        let videos = if !subcollection.videos.is_empty() {
            let videos = subcollection.videos
                .iter()
                .map(|video| {
                    let slug = &video.slug();
                    let title = video.title.as_deref().unwrap_or_else(|| video.slug());
                    let video_prefix = format!("{subcollection_slug}/{slug}/");
                    let video_thumbnail = thumbnail(video, &video_prefix, true);

                    formatdoc!(r#"
                        <a href="{subcollection_slug}/{slug}{index_suffix}">
                            {video_thumbnail}
                            <div>{title}</div>
                        </a>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            formatdoc!(r#"
                <div>
                    {videos}
                </div>
            "#)
        } else {
            String::new()
        };

        let subcollection_rendered = formatdoc!(r#"
            <h2>{title}</h2>

            {videos}

            <a href="{subcollection_slug}{index_suffix}">More from this collection</a>
        "#);

        body.push_str(&subcollection_rendered);
    }

    if outline.len() > 1 {
        let outline_items = outline
            .iter()
            .enumerate()
            .map(|(index, smartcrumb)| {
                let href = "../".repeat(outline.len() - 1 - index);
                smartcrumb.label(&href, site)
            })
            .collect::<Vec<String>>()
            .join("\n");

        body.push_str(&format!(r#"<div class="outline">{outline_items}</div>"#));
    }

    let root_prefix = if collection.path == "" { "".to_string() } else { "../".repeat(collection.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, outline);

    let collection_dir = context.build_dir.join(&collection.path);
    fs::create_dir_all(&collection_dir).unwrap();
    let index_path = collection_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
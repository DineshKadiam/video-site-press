use std::fs;

use indoc::formatdoc;

use crate::{Context, MarkdownSubset, Playlist, Site};
use crate::build::outline::Traversal;
use crate::build::widgets::{
    banner_html,
    layout,
    links,
    thumbnail,
    unlisted_badge
};

pub fn playlist_html(
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    traversal: &Traversal
) {
    let mut body = String::new();

    let title = if playlist.path == "" {
        playlist.title.as_deref().unwrap_or("Hyper 8")
    } else {
        playlist.title.as_deref().unwrap_or_else(|| playlist.slug())
    };

    // TODO: Hack to get margin to come out beneath header - replace with something proper soon
    body.push_str(r#"<div style="margin-top: 4rem;"></div>"#);

    if let Some(banner) = &playlist.banner {
        let playlist_prefix = "";
        let img = banner_html(banner, playlist_prefix);
        body.push_str(&img);
    }

    let unlisted = unlisted_badge(playlist.unlisted, &site.language.translations);

    body.push_str(&format!("<h1>{title} {unlisted}</h1>\n\n"));

    if let Some(description) = &playlist.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<p class="description">{description_html}</p>"#));
    }

    if !playlist.links.is_empty() {
        let links_rendered = links(&site.language, &playlist.links);
        body.push_str(&links_rendered);
    }

    let videos = playlist.public_videos_asc_by_release_date();

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .enumerate()
            .map(|(index, video)| {
                let traversal_with_video = traversal.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let number = index + 1;
                let label = video.label(&site.language.translations);
                let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
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
            <div class="playlist">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    // if !traversal.steps.is_empty() {
    //     let outline_items = traversal.steps
    //         .iter()
    //         .filter(|step| !step.is_playlist())
    //         .enumerate()
    //         .map(|(index, step)| {
    //             let prefix = "../".repeat(traversal.steps.len() - 1 - index);
    //             step.context_for_playlist(playlist, &prefix, site)
    //         })
    //         .collect::<Vec<String>>()
    //         .join("\n");

    //     body.push_str(&format!(r#"<div class="outline">{outline_items}</div>"#));
    // }

    let html = layout(&body, site, title, traversal);

    let playlist_dir = context.build_dir.join(&playlist.path);
    fs::create_dir_all(&playlist_dir).unwrap();
    let index_path = playlist_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
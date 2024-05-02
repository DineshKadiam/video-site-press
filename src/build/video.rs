use std::fs;
use std::sync::Arc;

use indoc::formatdoc;

use crate::{Context, Playlist, Site, Video};
use crate::build::Smartcrumb;
use crate::build::widgets::{copy_link, downloads, layout, player};
use crate::util::localized_date_d_mmmm_yyyy;

pub fn video_html(
    context: &Context,
    playlist: Option<&Playlist>,
    video: &Video,
    site: &Site,
    outline: &[Smartcrumb]
) {
    let description = if let Some(description) = &video.description { description } else { "" };
    let title = video.title.as_deref().unwrap_or(video.slug());

    let downloads_widget = downloads(site, video);

    let next_video = if video.unlisted {
        None
    } else {
        match playlist {
            Some(playlist) => {
                let listed_videos: Vec<Arc<Video>> = playlist.videos
                    .iter()
                    .filter(|video| !video.unlisted)
                    .map(|video| Arc::clone(video))
                    .collect();

                let active_position = listed_videos
                    .iter()
                    .position(|video_iterated| video_iterated.path == video.path)
                    .unwrap();

                listed_videos
                    .iter()
                    .nth(active_position + 1)
                    .map(|video| Arc::clone(video))
            }
            None => None
        }
    };

    let outline_items = outline
        .iter()
        .filter(|smartcrumb| !smartcrumb.is_video(video))
        .enumerate()
        .map(|(index, smartcrumb)| {
            let href = "../".repeat(outline.len() - 1 - index);

            if let Some(playlist_context) = smartcrumb.playlist_context(context, site, video) {
                playlist_context
            } else if outline.len() > 1 && index == outline.len() - 2 {
                if let Some(collection_context) = smartcrumb.collection_context(context, site, video) {
                    collection_context
                } else {
                    smartcrumb.label(&href, site)
                }
            } else {
                smartcrumb.label(&href, site)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    let outline_rendered = formatdoc!(r#"
        <div class="outline">
            {outline_items}
        </div>
    "#);

    let player_rendered = player(context, site, video, next_video);

    let copy_link_widget = copy_link(site, &video.path);

    let release_date = match video.release_date {
        Some(release_date) => {
            let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
            format!(r#"<span class="release_date">{formatted_date}</span>"#)
        }
        None => String::new()
    };

    let body = format!(r#"
        {player_rendered}
        <div class="video_info">
            <div class="align apart flex" style="margin-top: 2rem;">
                <div>
                    <h1 class="video_title">{title}</h1>
                    {release_date}
                </div>
                <div class="actions align flex space">
                    {copy_link_widget}
                    {downloads_widget}
                </div>
            </div>
            <p>
                {description}
            </p>
        </div>
        {outline_rendered}
    "#);

    let root_prefix = if video.path == "" { "".to_string() } else { "../".repeat(video.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, outline);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

use std::fs;
use std::sync::Arc;

use indoc::formatdoc;

use crate::{Context, MarkdownSubset, Playlist, Site, Video};
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_link, downloads, layout, links, player, unlisted_badge};
use crate::util::localized_date_d_mmmm_yyyy;

pub fn video_html(
    context: &Context,
    playlist: Option<&Playlist>,
    video: &Video,
    site: &Site,
    traversal: &Traversal
) {
    let description = if let Some(description) = &video.description {
        let description_html = MarkdownSubset::to_html(description);
        format!(r#"<p class="description">{description_html}</p>"#)
    } else {
        String::new()
    };
    let title = video.title.as_deref().unwrap_or(video.slug());

    let downloads_widget = downloads(site, video);

    let next_video = if video.unlisted {
        None
    } else {
        match playlist {
            Some(playlist) => {
                let videos = playlist.public_videos_asc_by_release_date();

                let active_position = videos
                    .iter()
                    .position(|video_iterated| video_iterated.path == video.path)
                    .unwrap();

                videos
                    .iter()
                    .nth(active_position + 1)
                    .map(|video| Arc::clone(video))
            }
            None => None
        }
    };

    let outline_items = traversal.steps
        .iter()
        .filter(|step| !step.is_video(video))
        .enumerate()
        .map(|(index, step)| {
            let prefix = "../".repeat(traversal.steps.len() - 1 - index);
            step.context_for_video(&prefix, site, video)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let outline_rendered = formatdoc!(r#"
        <div class="outline">
            {outline_items}
        </div>
    "#);

    let player_rendered = player(site, video, next_video);

    let copy_link_widget = copy_link(site, &video.path);

    let release_date = match video.release_date {
        Some(release_date) => {
            let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
            format!(r#"<div class="release_date">{formatted_date}</div>"#)
        }
        None => String::new()
    };

    let unlisted = unlisted_badge(video.unlisted, &site.language.translations);

    let links_rendered = links(&site.language, &video.links);

    let body = format!(r#"
        {player_rendered}
        <div class="video_info">
            <div class="align apart flex" style="margin-top: 2rem;">
                <div>
                    <h1 class="video_title">{title} {unlisted}</h1>
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
            {links_rendered}
        </div>
        {outline_rendered}
    "#);

    let html = layout(&body, site, title, traversal);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

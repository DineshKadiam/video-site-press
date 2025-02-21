// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use crate::{Context, MarkdownSubset, Playlist, Site, Video};
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_link, downloads, layout, links, player, unlisted_badge};
use crate::icons;
use crate::util::{
    html_escape_outside_attribute,
    localized_date_d_mmmm_yyyy
};

use super::MergedOptions;

pub fn video_html(
    context: &Context,
    merged_options: &MergedOptions,
    playlist: Option<&Playlist>,
    site: &Site,
    traversal: &Traversal,
    video: &Video
) {
    let index_suffix = site.index_suffix();

    let description = if let Some(description) = &video.description {
        let description_html = MarkdownSubset::to_html(description);
        format!(r#"<div class="description reading_width">{description_html}</div>"#)
    } else {
        String::new()
    };

    let title = match &video.title {
        Some(title) => title,
        None => if video.path == "" { "Hyper 8" } else { video.slug() }
    };
    let e_title = html_escape_outside_attribute(title);

    let r_downloads = match merged_options.download {
        true => downloads(site, video),
        false => String::new()
    };

    let next_video = if video.unlisted {
        None
    } else {
        match playlist {
            Some(playlist) => {
                let videos = playlist.public_videos_in_playlist_order();

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

    // let outline_items = traversal.steps
    //     .iter()
    //     .filter(|step| !step.is_video(video))
    //     .enumerate()
    //     .map(|(index, step)| {
    //         let prefix = "../".repeat(traversal.steps.len() - 1 - index);
    //         step.context_for_video(&prefix, site, video)
    //     })
    //     .collect::<Vec<String>>()
    //     .join("\n");

    // let outline_rendered = formatdoc!(r#"
    //     <div class="outline">
    //         {outline_items}
    //     </div>
    // "#);

    let video_prefix = "";
    let player_rendered = player(next_video, site, video, video_prefix);

    let r_copy_link = if merged_options.copy_link {
        copy_link(site, &video.path)
    } else {
        String::new()
    };

    let r_embed = if merged_options.embedding && site.base_url.is_some() {
        let embed_icon = icons::embed();

        let t_embed = &site.language.translations.embed;
        format!(r#"
            <div>
                <a href="embed-codes{index_suffix}">
                    <span class="icon">{embed_icon}</span> {t_embed}
                </a>
            </div>
        "#)
    } else {
        String::new()
    };

    let release_date = match video.release_date {
        Some(release_date) => {
            let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
            format!(r#"<div class="release_date">{formatted_date}</div>"#)
        }
        None => String::new()
    };

    let unlisted = unlisted_badge(merged_options.unlisted, &site.language.translations);

    let links_rendered = links(&site.language, &video.links);

    let body = format!(r#"
        {player_rendered}
        <div class="video_info">
            <div class="align apart flex" style="margin-top: 2rem;">
                <div>
                    <h1 class="video_title">{e_title} {unlisted}</h1>
                    {release_date}
                </div>
                <div class="actions align flex space">
                    {r_copy_link}
                    {r_embed}
                    {r_downloads}
                </div>
            </div>
            {description}
            {links_rendered}
        </div>
    "#);

    let html = layout(&body, site, title, traversal);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

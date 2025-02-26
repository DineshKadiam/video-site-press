// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    MarkdownSubset,
    Playlist,
    Site
};
use crate::build::outline::Traversal;
use crate::util::localized_date_d_mmmm_yyyy;
use crate::build::Layout;
use crate::build::widgets::{
    banner_html,
    links,
    thumbnail,
    unlisted_badge
};
use crate::util::html_escape_outside_attribute;

use super::MergedOptions;

/// Generates and writes a collection index page to the build directory.
pub fn collection_html(
    context: &Context,
    collection: &Collection,
    merged_options: &MergedOptions,
    site: &Site,
    traversal: &Traversal
) {
    let translations = &site.language.translations;

    let mut body = String::new();

    let title = match &collection.title {
        Some(title) => title,
        None => if collection.path.is_empty() { "Hyper 8" } else { collection.slug() }
    };
    let e_title = html_escape_outside_attribute(title);

    // TODO: Hack to get margin to come out beneath header - replace with something proper soon
    body.push_str(r#"<div style="margin-top: 4rem;"></div>"#);

    if let Some(banner) = &collection.banner {
        let collection_prefix = "";
        let img = banner_html(banner, collection_prefix);
        body.push_str(&img);
    }

    let unlisted = unlisted_badge(merged_options.unlisted, translations);

    body.push_str(&format!("<h1>{e_title} {unlisted}</h1>\n\n"));

    if let Some(description) = &collection.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<div class="description reading_width">{description_html}</div>"#));
    }

    if !collection.links.is_empty() {
        let links_rendered = links(&site.language, &collection.links);
        body.push_str(&links_rendered);
    }

    let videos = collection.public_videos_in_collection_video_order();

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let e_title = match &video.title {
                    Some(title) => html_escape_outside_attribute(title),
                    None => video.slug().to_string()
                };

                let video_thumbnail = thumbnail(translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        {video_thumbnail}
                        <div class="caption">{e_title}</div>
                        {release_date}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let videos_rendered = formatdoc!(r#"
            <div class="videos_grid">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    let playlists = collection.public_playlists_desc_by_release_date();
    for playlist in &playlists {
        let rendered = render_playlist(playlist, site, traversal);
        body.push_str(&rendered);
    }

    let subcollections = collection.public_subcollections_desc_by_release_date();
    for subcollection in &subcollections {
        let rendered = render_collection(subcollection, site, traversal);
        body.push_str(&rendered);
    }

    // if traversal.steps.len() > 1 {
    //     let outline_items = traversal.steps
    //         .iter()
    //         .filter(|step| !step.is_collection(collection))
    //         .enumerate()
    //         .map(|(index, step)| {
    //             let prefix = "../".repeat(traversal.steps.len() - 1 - index);
    //             step.context_for_collection(collection, &prefix, site)
    //         })
    //         .collect::<Vec<String>>()
    //         .join("\n");

    //     body.push_str(&format!(r#"<div class="outline">{outline_items}</div>"#));
    // }

    let layout = Layout::new();
    let html = layout.render(&body, site, title, traversal);

    let collection_dir = context.build_dir.join(&collection.path);
    fs::create_dir_all(&collection_dir).unwrap();
    let index_path = collection_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

fn render_collection(
    collection: &Collection,
    site: &Site,
    traversal: &Traversal
) -> String {
    let mut output = String::new();

    let traversal_with_collection = traversal.peek_collection(collection);
    let traversal_with_collection_prefix = traversal_with_collection.file_prefix_desc();

    let collection_href = traversal_with_collection.index_desc(site);

    // TODO: Here and elsewhere - are we using the alt text already here? Probably not?
    if let Some(banner) = &collection.banner {
        let banner = banner_html(banner, &traversal_with_collection_prefix);

        let wrapped = format!(r#"<a href="{collection_href}">{banner}</a>"#);

        output.push_str(&wrapped);
    }

    let videos = collection.public_videos_in_collection_video_order();

     if !videos.is_empty() {
        let videos = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal_with_collection.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let e_title = match &video.title {
                    Some(title) => html_escape_outside_attribute(title),
                    None => video.slug().to_string()
                };
                let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        {video_thumbnail}
                        <div class="caption">{e_title}</div>
                        {release_date}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let breadcrumbs = traversal_with_collection.peeked_parents(site);
        let breadcrumbs_rendered = if breadcrumbs.is_empty() {
            String::new()
        } else {
            let rendered = breadcrumbs
                .iter()
                .map(|breadcrumb| breadcrumb.step.breadcrumb(&breadcrumb.href, site))
                .collect::<Vec<String>>()
                .join(" › ");

            format!(r#"<div class="parents">{rendered}</div>"#)
        };

        let e_title = match &collection.title {
            Some(title) => html_escape_outside_attribute(title),
            None => collection.slug().to_string()
        };
        let collection_title_and_videos = formatdoc!(r#"
            <h2>
                <a href="{collection_href}">
                    {e_title}
                </a>
                {breadcrumbs_rendered}
            </h2>

            <div class="videos_grid">
                {videos}
            </div>
        "#);

        output.push_str(&collection_title_and_videos);
    }

    let playlists = collection.public_playlists_desc_by_release_date();
    for playlist in &playlists {
        let rendered = render_playlist(playlist, site, &traversal_with_collection);
        output.push_str(&rendered);
    }

    let subcollections = collection.public_subcollections_desc_by_release_date();
    for subcollection in &subcollections {
        let rendered = render_collection(subcollection, site, &traversal_with_collection);
        output.push_str(&rendered);
    }

    output
}

fn render_playlist(
    playlist: &Playlist,
    site: &Site,
    traversal: &Traversal
) -> String {
    let traversal_with_playlist = traversal.peek_playlist(playlist);
    let traversal_with_playlist_prefix = traversal_with_playlist.file_prefix_desc();

    let banner = if let Some(banner) = &playlist.banner {
        banner_html(banner, &traversal_with_playlist_prefix)
    } else {
        String::new()
    };

    let videos = playlist.public_videos_in_playlist_order()
        .iter()
        .enumerate()
        .map(|(index, video)| {
            let traversal_with_video = traversal_with_playlist.peek_video(video);
            let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

            let release_date = match video.release_date {
                Some(release_date) => {
                    let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                    format!(r#"<span class="release_date">{formatted_date}</span>"#)
                }
                None => String::new()
            };
            let number = index + 1;
            let e_title = match &video.title {
                Some(title) => html_escape_outside_attribute(title),
                None => video.slug().to_string()
            };

            let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

            let video_href = traversal_with_video.index_desc(site);
            formatdoc!(r#"
                <a href="{video_href}">
                    {video_thumbnail}
                    <div class="caption">{e_title}</div>
                    {release_date} <span class="number"># {number}</span>
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let videos_rendered = formatdoc!(r#"
        <div class="videos_grid">
            {videos}
        </div>
    "#);

    let breadcrumbs = traversal_with_playlist.peeked_parents(site);
    let breadcrumbs_rendered = if breadcrumbs.is_empty() {
        String::new()
    } else {
        let rendered = breadcrumbs
            .iter()
            .map(|breadcrumb| breadcrumb.step.breadcrumb(&breadcrumb.href, site))
            .collect::<Vec<String>>()
            .join(" › ");

        format!(r#"<div class="parents">{rendered}</div>"#)
    };

    let e_title = match &playlist.title {
        Some(title) => html_escape_outside_attribute(title),
        None => playlist.slug().to_string()
    };
    let playlist_href = traversal_with_playlist.index_desc(site);
    formatdoc!(r#"
        <h2>
            <a href="{playlist_href}">
                {e_title}
            </a>
            {breadcrumbs_rendered}
        </h2>
        <a href="{playlist_href}">{banner}</a>
        {videos_rendered}
    "#)
}
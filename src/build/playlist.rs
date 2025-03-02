// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Context,
    MarkdownSubset,
    OpenGraph,
    OpenGraphMeta,
    Playlist,
    Site
};
use crate::build::{AssetHashes, Layout};
use crate::build::outline::Traversal;
use crate::build::widgets::{
    banner_html,
    links,
    thumbnail,
    unlisted_badge
};
use crate::util::{
    html_escape_outside_attribute,
    localized_date_d_mmmm_yyyy
};

pub fn playlist_html(
    asset_hashes: &AssetHashes,
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    traversal: &Traversal
) {
    let mut layout = Layout::new();
    let translations = &site.language.translations;

    let mut body = String::new();

    let title = match &playlist.title {
        Some(title) => title,
        None => if playlist.path.is_empty() { "Hyper 8" } else { playlist.slug() }
    };
    let e_title = html_escape_outside_attribute(title);

    // TODO: Hack to get margin to come out beneath header - replace with something proper soon
    body.push_str(r#"<div style="margin-top: 4rem;"></div>"#);

    if let Some(banner) = &playlist.banner {
        let playlist_prefix = "";
        let img = banner_html(banner, playlist_prefix);
        body.push_str(&img);
    }

    let r_unlisted_badge = if playlist.unlisted() {
        layout.no_indexing();
        unlisted_badge(&site.language.translations)
    } else {
        String::new()
    };

    body.push_str(&format!("<h1>{e_title} {r_unlisted_badge}</h1>\n\n"));

    if let Some(description) = &playlist.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<div class="description">{description_html}</div>"#));
    }

    if !playlist.links.is_empty() {
        let links_rendered = links(&site.language, &playlist.links);
        body.push_str(&links_rendered);
    }

    let videos = playlist.public_videos_in_playlist_order();

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .enumerate()
            .map(|(index, video)| {
                let traversal_with_video = traversal.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let number = index + 1;
                let label = video.label(translations);
                let video_thumbnail = thumbnail(translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        <div>{video_thumbnail}</div>
                        <div>
                            <div class="caption">{label}</div>
                            {release_date} <span class="number"># {number}</span>
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

    if let OpenGraph::LinkPreviews = playlist.opengraph() {
        if let Some(playlist_url) = site.absolute_url(&playlist.path) {
            let mut meta = OpenGraphMeta::new(title.to_string(), playlist_url);

            if let Some(description) = &playlist.description {
                let plaintext = MarkdownSubset::to_plaintext(description);

                // As a somewhat arbitrary practical decision we currently
                // only use "our" description as the opengraph description
                // when it is less than 256 characters long in plaintext.
                if plaintext.len() < 256 {
                    meta.description(description);
                }
            }

            // TODO: Technically working but we're keeping it disabled until
            // banners are actually integrated in the visible frontend as
            // well.
            // if let Some(banner) = &playlist.banner {
            //     if let Some(poster_assets) = &banner.assets {
            //         let image_url_prefix = site.absolute_url_prefix_unchecked(&playlist.path);
            //         let opengraph_image = poster_assets.opengraph_image("banner", &image_url_prefix);

            //         meta.image(opengraph_image);

            //         if let Some(description) = &banner.description {
            //             meta.image_alt(description);
            //         }
            //     }
            // }

            layout.add_opengraph_meta(meta);
        }
    }

    let html = layout.render(
        asset_hashes,
        &body,
        site,
        title,
        traversal
    );

    let playlist_dir = context.build_dir.join(&playlist.path);
    fs::create_dir_all(&playlist_dir).unwrap();
    let index_path = playlist_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use hyper8_util::html_escape_outside_attribute;

use crate::{
    Build,
    Context,
    MarkdownSubset,
    PlatformIntegrationMeta,
    Playlist,
    Site
};
use crate::build::Layout;
use crate::build::outline::Traversal;
use crate::build::widgets::{
    banner_html,
    bubbles,
    links,
    subscribe_button,
    thumbnail,
    unlisted_badge
};
use crate::util::localized_date_d_mmmm_yyyy;

pub fn playlist_html(
    build: &Build,
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    subscribe_permalink: Option<String>,
    traversal: &Traversal
) {
    let mut layout = Layout::new();

    let language = &site.language;
    let translations = &language.translations;

    let mut body = String::new();

    let title = playlist.title_or_slug_or_generic_label(translations);
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
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let videos = playlist.public_videos_in_playlist_order(language);

    let video_bubbles = bubbles("", &videos);
    let heading = formatdoc!(r#"
        <div class="container_title">
            <h1>{e_title} {r_unlisted_badge}</h1>
            {video_bubbles}
        </div>
    "#);
    body.push_str(&heading);

    if let Some(description) = &playlist.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<div class="description">{description_html}</div>"#));
    }

    if let Some(subscribe_permalink) = &subscribe_permalink {
        body.push_str(&subscribe_button(subscribe_permalink, site));
        layout.add_feed_links();
    }

    if !playlist.links.is_empty() {
        let links_rendered = links(&site.language, &playlist.links);
        body.push_str(&links_rendered);
    }

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let label = video.title_or_slug_or_generic_label(translations);
                let video_thumbnail = thumbnail(translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        <div>{video_thumbnail}</div>
                        <div>
                            <div class="caption">{label}</div>
                            {release_date}
                        </div>
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let videos_rendered = formatdoc!(r#"
            <div class="playlist_page">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    if playlist.platform_integration().enabled() {
        if let Some(playlist_url) = site.absolute_url_index(&playlist.site_path) {
            let mut meta = PlatformIntegrationMeta::new(title.to_string(), playlist_url);

            if let Some(description) = &playlist.description {
                let description_plaintext = MarkdownSubset::to_plaintext(description);

                // As a somewhat arbitrary practical decision we currently
                // only use "our" description as the description for platform integration
                // when it is less than 256 characters long in plaintext.
                if description_plaintext.len() < 256 {
                    meta.description(&description_plaintext);
                }
            }

            // TODO: Technically working but we're keeping it disabled until
            // banners are actually integrated in the visible frontend as
            // well.
            // if let Some(banner) = &playlist.banner {
            //     if let Some(poster_assets) = &banner.assets {
            //         let image_url_prefix = site.absolute_url_prefix_unchecked(&playlist.site_path);
            //         let platform_integration_image = poster_assets.platform_integration_image("banner", &image_url_prefix);

            //         meta.image(platform_integration_image);

            //         if let Some(description) = &banner.description {
            //             meta.image_alt(description);
            //         }
            //     }
            // }

            layout.add_platform_integration_meta(meta);
        }
    }

    let html = layout.render(
        build,
        &body,
        site,
        title,
        traversal
    );

    let playlist_dir = context.build_dir.join(playlist.site_path.filesystem_relative());
    fs::create_dir_all(&playlist_dir).unwrap();
    let index_path = playlist_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
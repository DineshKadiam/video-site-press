// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use hyper8_util::AnchorIdGenerator;
use hyper8_util::html_escape_outside_attribute;

use crate::{
    Build,
    Collection,
    Container,
    Context,
    MarkdownSubset,
    PlatformIntegrationMeta,
    Playlist,
    Site
};
use crate::build::outline::Traversal;
use crate::build::Layout;
use crate::build::widgets::{
    banner_html,
    bubbles,
    link_button,
    subscribe_button,
    thumbnail,
    unlisted_badge
};
use crate::util::localized_date_d_mmmm_yyyy;

/// Generates and writes a collection index page to the build directory.
pub fn collection_html(
    build: &Build,
    context: &Context,
    collection: &Collection,
    site: &Site,
    subscribe_permalink: Option<String>,
    traversal: &Traversal
) {
    let mut anchor_id_generator = AnchorIdGenerator::new();
    let mut layout = Layout::new();

    let language = &site.language;
    let translations = &language.translations;

    let mut body = String::new();

    let title = collection.title_or_slug_or_generic_label(translations);
    let e_title = html_escape_outside_attribute(title);

    // TODO: Hack to get margin to come out beneath header - replace with something proper soon
    body.push_str(r#"<div style="margin-top: 4rem;"></div>"#);

    if let Some(banner) = &collection.banner {
        let collection_prefix = "";
        let img = banner_html(banner, collection_prefix);
        body.push_str(&img);
    }

    let r_unlisted_badge = if collection.unlisted() {
        layout.no_indexing();
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let videos = collection.public_videos_in_collection_video_order(language);

    let video_bubbles = bubbles("", &videos);
    let heading = formatdoc!(r#"
        <div class="container_title">
            <h1>{e_title} {r_unlisted_badge}</h1>
            {video_bubbles}
        </div>
    "#);
    body.push_str(&heading);

    if let Some(description) = &collection.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<div class="description reading_width">{description_html}</div>"#));
    }

    let mut actions = Vec::new();

    if let Some(subscribe_permalink) = &subscribe_permalink {
        let subscribe_button = subscribe_button(subscribe_permalink, site);

        actions.push(subscribe_button);
        layout.add_feed_links();
    }

    for link in &collection.links {
        let link_button = link_button(link, translations);
        actions.push(link_button);
    }

    if !actions.is_empty() {
        let actions = actions.join("");
        body.push_str(&format!(r#"<div class="actions pad">{actions}</div>"#));
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
                let e_title = match &video.title {
                    Some(title) => html_escape_outside_attribute(title),
                    None => video.slug_unchecked().to_string()
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

    let containers = collection.public_containers_in_container_order(language);

    for container in &containers {
        let container_rendered = match container {
            Container::Collection(collection) => {
                render_collection(
                    &mut anchor_id_generator,
                    collection,
                    site,
                    traversal
                )
            }
            Container::Playlist(playlist) => {
                render_playlist(
                    &mut anchor_id_generator,
                    playlist,
                    site,
                    traversal
                )
            }
        };

        body.push_str(&container_rendered);
    }

    if collection.platform_integration().enabled() {
        if let Some(collection_url) = site.absolute_url_index(&collection.site_path) {
            let mut meta = PlatformIntegrationMeta::new(title.to_string(), collection_url);

            if let Some(description) = &collection.description {
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
            // if let Some(banner) = &collection.banner {
            //     if let Some(poster_assets) = &banner.assets {
            //         let image_url_prefix = site.absolute_url_prefix_unchecked(&collection.site_path);
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

    let collection_dir = context.build_dir.join(collection.site_path.filesystem_relative());
    fs::create_dir_all(&collection_dir).unwrap();
    let index_path = collection_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

fn render_collection(
    anchor_id_generator: &mut AnchorIdGenerator,
    collection: &Collection,
    site: &Site,
    traversal: &Traversal
) -> String {
    let mut output = String::new();

    let language = &site.language;
    let translations = &language.translations;

    let traversal_with_collection = traversal.peek_collection(collection);
    let traversal_with_collection_prefix = traversal_with_collection.prefix_desc();

    let collection_href = traversal_with_collection.index_desc(site);

    // TODO: Here and elsewhere - are we using the alt text already here? Probably not?
    if let Some(banner) = &collection.banner {
        let banner = banner_html(banner, &traversal_with_collection_prefix);

        let wrapped = format!(r#"<a href="{collection_href}">{banner}</a>"#);

        output.push_str(&wrapped);
    }

    let videos = collection.public_videos_in_collection_video_order(language);

     if !videos.is_empty() {
        let videos_rendered = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal_with_collection.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let e_title = match &video.title {
                    Some(title) => html_escape_outside_attribute(title),
                    None => video.slug_unchecked().to_string()
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

        let slug = collection.slug_unchecked();

        let id = anchor_id_generator.generate_for_slug(slug);

        let e_title = match &collection.title {
            Some(title) => html_escape_outside_attribute(title),
            None => slug.to_string()
        };

        let video_bubbles = bubbles(&traversal_with_collection_prefix, &videos);
        let collection_title_and_videos = formatdoc!(r##"
            <div class="container_title" id="{id}">
                <h2>
                    <a href="{collection_href}">
                        {e_title}
                    </a>
                    <a class="anchor" href="#{id}">#</a>
                    {breadcrumbs_rendered}
                </h2>
                {video_bubbles}
            </div>

            <div class="videos_grid">
                {videos_rendered}
            </div>
        "##);

        output.push_str(&collection_title_and_videos);
    }

    let containers = collection.public_containers_in_container_order(language);

    for container in &containers {
        let container_rendered = match container {
            Container::Collection(collection) => {
                render_collection(
                    anchor_id_generator,
                    collection,
                    site,
                    &traversal_with_collection
                )
            }
            Container::Playlist(playlist) => {
                render_playlist(
                    anchor_id_generator,
                    playlist,
                    site,
                    &traversal_with_collection
                )
            }
        };

        output.push_str(&container_rendered);
    }

    output
}

fn render_playlist(
    anchor_id_generator: &mut AnchorIdGenerator,
    playlist: &Playlist,
    site: &Site,
    traversal: &Traversal
) -> String {
    let language = &site.language;
    let translations = &language.translations;

    let traversal_with_playlist = traversal.peek_playlist(playlist);
    let traversal_with_playlist_prefix = traversal_with_playlist.prefix_desc();

    let banner = if let Some(banner) = &playlist.banner {
        banner_html(banner, &traversal_with_playlist_prefix)
    } else {
        String::new()
    };

    let videos = playlist.public_videos_in_playlist_order(language);

    let videos_rendered = videos
        .iter()
        .map(|video| {
            let traversal_with_video = traversal_with_playlist.peek_video(video);
            let traversal_with_video_prefix = traversal_with_video.prefix_desc();

            let release_date = match video.release_date {
                Some(release_date) => {
                    let formatted_date = localized_date_d_mmmm_yyyy(release_date, language.chrono_locale);
                    format!(r#"<span class="release_date">{formatted_date}</span>"#)
                }
                None => String::new()
            };
            let e_title = match &video.title {
                Some(title) => html_escape_outside_attribute(title),
                None => video.slug_unchecked().to_string()
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
            {videos_rendered}
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

    let playlist_href = traversal_with_playlist.index_desc(site);

    let slug = playlist.slug_unchecked();

    let id = anchor_id_generator.generate_for_slug(slug);

    let e_title = match &playlist.title {
        Some(title) => html_escape_outside_attribute(title),
        None => slug.to_string()
    };

    let video_bubbles = bubbles(&traversal_with_playlist_prefix, &videos);
    formatdoc!(r##"
        <div class="container_title" id="{id}">
            <h2>
                <a href="{playlist_href}">
                    {e_title}
                </a>
                <a class="anchor" href="#{id}">#</a>
                {breadcrumbs_rendered}
            </h2>
            {video_bubbles}
        </div>
        <a href="{playlist_href}">{banner}</a>
        {videos_rendered}
    "##)
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! This module contains the building blocks for generating the plain rss,
//! media rss and podcast rss feeds. Each of those respective modules calls
//! this module's functions to render the shared base markup, while adding
//! specific extension markup through closures operating at the channel and
//! item level.
//!
//! RSS 2.0 specification for reference:
//! https://www.rssboard.org/rss-specification

use std::sync::Arc;

use chrono::{DateTime, Utc};
use indoc::formatdoc;

use crate::{MarkdownSubset, Site, Video};
use crate::build::GENERATOR_INFO;
use crate::util::html_escape_outside_attribute;

pub fn rss(
    build_time: DateTime<Utc>,
    // Specific channel-level extension markup added by the caller (e.g. media
    // rss or podcast rss specific markup).
    channel_extensions: &str,
    description: &Option<String>,
    // E.g. to pass specific media or podcast namespaces
    extra_namespaces: &[&str],
    // Specific item-level extension markup added by the caller (e.g. media
    // rss or podcast rss specific markup).
    feed_url: &str,
    item_extensions: &mut impl FnMut(DateTime<Utc>, &Site, &Video) -> String,
    site: &Site,
    title: &str,
    videos: &[Arc<Video>]
) -> String {
    let items = videos
        .iter()
        .map(|video| {
            item(
                build_time,
                item_extensions,
                site,
                video
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let description = if let Some(description) = description {
        let description_html = MarkdownSubset::to_html(description);
        html_escape_outside_attribute(&description_html)
    } else {
        // TODO: Eventually find something better to fallback to.
        // Note that this is a mandatory field in RSS (https://www.rssboard.org/rss-specification#requiredChannelElements)
        format!("Hyper 8 Video System {}", env!("HYPER8_VERSION_DISPLAY"))
    };

    let title_escaped = html_escape_outside_attribute(title);

    let link = site.absolute_url_unchecked();

    // TODO: Use banner eventually
    // let image = if let Some(banner) = banner {
    //     let url = base_url.join_file(format!("{filename}?{hash}"));
    //     let title = match &banner.description {
    //         Some(description) => html_escape_outside_attribute(description),
    //         None => title_escaped.clone()
    //     };

    //     formatdoc!(r#"
    //         <image>
    //             <height>{edge_size}</height>
    //             <link>{link}</link>
    //             <title>{title}</title>
    //             <url>{url}</url>
    //             <width>{edge_size}</width>
    //         </image>
    //     "#)
    // } else {
    //     String::new()
    // };

    let build_time_rfc2822 = build_time.to_rfc2822();
    let language = &site.language.code;

    let extra_namespaces = extra_namespaces.join(" ");

    // Note that atom:link inside <channel> is not something that points to an
    // atom feed (we use it to point back to an RSS feed after all), but
    // instead just a piece of functionality we borrow from the atom
    // specification to include a link to the feed itself inside an RSS feed
    // (which natively does not offer such functionality). See also
    // https://www.rssboard.org/rss-profile#namespace-elements-atom-link
    formatdoc!(r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" {extra_namespaces}>
            <channel>
                <atom:link href="{feed_url}" rel="self" type="application/rss+xml"/>
                <description>{description}</description>
                <generator>{GENERATOR_INFO}</generator>
                <language>{language}</language>
                <lastBuildDate>{build_time_rfc2822}</lastBuildDate>
                <link>{link}</link>
                <title>{title_escaped}</title>
                {channel_extensions}
                {items}
            </channel>
        </rss>
    "#)
}

fn item(
    build_time: DateTime<Utc>,
    // Specific item-level extension markup added by the caller (e.g. media
    // rss or podcast rss specific markup).
    item_extensions: &mut impl FnMut(DateTime<Utc>, &Site, &Video) -> String,
    site: &Site,
    video: &Video
) -> String {
    let description = if let Some(description) = &video.description {
        let description_html = MarkdownSubset::to_html(description);
        let description_escaped = html_escape_outside_attribute(&description_html);
        format!("<description>{description_escaped}</description>")
    } else {
        String::new()
    };

    let link = site.absolute_url_index_unchecked(&video.site_path);

    let title_escaped = html_escape_outside_attribute(video.title_or_slug_unchecked());

    // Execute closure that may add e.g. media rss or podcast rss specific
    // markup.
    let extensions = item_extensions(build_time, site, video);

    formatdoc!(r#"
        <item>
            {description}
            <guid>{link}</guid>
            <link>{link}</link>
            <title>{title_escaped}</title>
            {extensions}
        </item>
    "#)
}

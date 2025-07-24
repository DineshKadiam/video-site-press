// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Video podcasts reference from apple:
//! https://podcasters.apple.com/support/3684-video-podcasts (general info)
//! https://podcasters.apple.com/support/823-podcast-requirements (technical requirements)
//!
//! Specification from the Podcast Standards Project as reference:
//! https://github.com/Podcast-Standards-Project/PSP-1-Podcast-RSS-Specification
//!
//! "A Podcaster’s Guide to RSS" from apple as reference:
//! https://help.apple.com/itc/podcasts_connect/#/itcb54353390

use std::fs;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    Context,
    Site,
    SitePath,
    Video
};

use super::PODCAST_RSS_FILENAME;
use super::rss::rss;

// Used to compute the podcast's guid.
// See https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md#guid
const PODCAST_NAMESPACE_UUID: &str = "ead4c236-bf58-58c6-a2c6-a6b28d128cb6";

pub fn item_extensions(
    _build_time: DateTime<Utc>,
    site: &Site,
    video: &Video
) -> String {
    let mut extensions = Vec::new();

    // Podcast RSS items without media/enclosure are probably not
    // standard-compliant and we should anyhow rather filter them out of the
    // feed entirely if we can not cleanly represent a "video with the actual
    // video content yet to come". For now we do it like this just to handle
    // these edge cases in a crash-free manner.
    if !video.versions.is_empty() {
        // <enclosure …>

        let video_file = video.recommended_version().unwrap();

        let url = site.absolute_url_file_unchecked(&video_file.file_name, &video.site_path);

        let mime_type = video_file.mime_type();

        let filesize_bytes = video_file.file_meta.size;

        let enclosure = format!(r#"<enclosure length="{filesize_bytes}" type="{mime_type}" url="{url}"/>"#);

        extensions.push(enclosure);

        // <itunes:duration …>

        let duration_seconds = video_file.video_meta.as_ref().unwrap().duration;
        let itunes_duration = format!(r#"<itunes:duration>{duration_seconds}</itunes:duration>"#);
        extensions.push(itunes_duration);
    }


    // <itunes:image …>

    if let Some(poster) = &video.poster {
        if let Some(poster_assets) = &poster.assets {
            // According to apple's specification "Artwork must be a minimum size
            // of 1400 x 1400 pixels and a maximum size of 3000 x 3000 pixels, in
            // JPEG or PNG format".
            let target_filename = poster_assets.largest().target_filename("poster");
            let image_url = site.absolute_url_file_unchecked(&target_filename, &video.site_path);
            let itunes_image = format!(r#"<itunes:image href="{image_url}"/>"#);

            extensions.push(itunes_image);
        }
    }

    // TODO: Consider further items for future implementation:
    // https://github.com/Podcast-Standards-Project/PSP-1-Podcast-RSS-Specification?tab=readme-ov-file#required-item-elements

    extensions.join("\n")
}

pub fn podcast_rss(
    build_time: DateTime<Utc>,
    context: &Context,
    description: &Option<String>,
    site: &Site,
    site_path: &SitePath,
    title: &str,
    videos: &[Arc<Video>]
) {
    let feed_url = site.absolute_url_file_unchecked(PODCAST_RSS_FILENAME, site_path);

    let mut extensions = Vec::new();

    // TODO: Use banner image as itunes:image
    // if let Some(banner) = banner {
    //     let itunes_image = format!(r#"<itunes:image href="{url}"/>"#);
    //     extensions.push(itunes_image);
    // }

    let namespace_uuid = Uuid::parse_str(PODCAST_NAMESPACE_UUID).unwrap();
    let normalized_url = site.base_url.as_ref().unwrap().without_scheme_and_trailing_slash();
    let guid = Uuid::new_v5(&namespace_uuid, normalized_url.as_bytes());
    let podcast_guid = format!(r#"<podcast:guid>{guid}</podcast:guid>"#);
    extensions.push(podcast_guid);

    // TODO: Consider futher items for future implementation:
    // https://github.com/Podcast-Standards-Project/PSP-1-Podcast-RSS-Specification?tab=readme-ov-file#required-channel-elements

    let channel_extensions = extensions.join("\n");

    let extra_namespaces = &[
        r#"xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd""#,
        r#"xmlns:podcast="https://podcastindex.org/namespace/1.0""#
    ];

    let xml = rss(
        build_time,
        &channel_extensions,
        description,
        extra_namespaces,
        &feed_url,
        &mut item_extensions,
        site,
        title,
        videos
    );

    let path = context.build_dir.join(site_path.filesystem_relative()).join(PODCAST_RSS_FILENAME);
    fs::write(path, xml).unwrap();
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Media RSS 1.5.1 Specification for reference:
//! https://www.rssboard.org/media-rss

use std::fs;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use indoc::formatdoc;

use crate::{
    Context,
    Site,
    SitePath,
    Video
};

use super::MEDIA_RSS_FILENAME;
use super::rss::rss;

pub fn item_extensions(
    _build_time: DateTime<Utc>,
    site: &Site,
    video: &Video
) -> String {
    let mut extensions = Vec::new();

    // <media:group …>

    if !video.versions.is_empty() {
        let recommended_version = video.recommended_version().unwrap();

        let mut media_content_tags = Vec::new();

        // <media:content …>

        for video_file in &video.versions {
            let url = site.absolute_url_file_unchecked(&video_file.file_name, &video.site_path);
            // TODO: Implement "audio" when we work on audio-only formats
            let medium = "video";
            let mime_type = video_file.mime_type();
            let filesize_bytes = video_file.file_meta.size;
            let is_default = match video_file == recommended_version {
                true => r#"isDefault="true""#,
                false => ""
            };

            let video_meta = video_file.video_meta.as_ref().unwrap();

            let duration = video_meta.duration;
            let frame_rate = video_meta.video_frame_rate;
            let height = video_meta.height;
            let width = video_meta.width;

            // TODO: Attributes we could add in the future:
            // - bitrate
            // - channels
            // - lang
            // - samplingrate
            // (see https://www.rssboard.org/media-rss#media-content)

            let media_content = formatdoc!(r#"
                <media:content
                    duration="{duration}"
                    fileSize="{filesize_bytes}"
                    framerate="{frame_rate}"
                    height="{height}"
                    {is_default}
                    medium="{medium}"
                    type="{mime_type}"
                    url="{url}"
                    width="{width}"/>
            "#);

            media_content_tags.push(media_content);
        }

        let media_content_joined = media_content_tags.join("\n");
        let media_group = formatdoc!(r#"
            <media:group>
                {media_content_joined}
            </media:group>
        "#);

        extensions.push(media_group);
    }

    // TODO: If we implement "explicit" as a datapoint in the data model
    // to use it as an attribute for podcast rss, we could introduce <media:rating>
    // here as a bonus outcome:
    // <media:rating scheme="urn:simple">adult</media:rating>
    // <media:rating scheme="urn:simple">nonadult</media:rating>
    // (see https://www.rssboard.org/media-rss#media-rating)

    // TODO: We could implement <media:player> here and refer to our embeddable player url.
    // We would need to decide whether we want to include this only if an embeddable player
    // is rendered by us for some reason already (platform integration, explicit embeds),
    // or whether we want to implicitly render the player even just to offer it via media rss
    // as well.
    // (see https://www.rssboard.org/media-rss#media-player)

    // <media:thumbnail …>

    if let Some(poster) = &video.poster {
        if let Some(poster_assets) = &poster.assets {
            let largest = poster_assets.largest();
            let height = largest.height;
            let width = largest.width;
            let image_filename = format!("poster_{width}x{height}.jpg");
            let image_url = site.absolute_url_file_unchecked(&image_filename, &video.site_path);
            let media_thumbnail = format!(r#"<media:thumbnail height="{height}" url="{image_url}" width="{width}"/>"#);

            extensions.push(media_thumbnail);
        }
    }

    // TODO: Use media:text for subtitles/transcripts?
    // (see https://www.rssboard.org/media-rss#media-text)

    // TODO: In addition/as alternative to <media:player> we could use media:embed
    // to offer an embeddable player widget.
    // (see https://www.rssboard.org/media-rss#media-embed)

    // TODO: Possibly include media:license if we ever add dedicated licensing information
    // in the datamodel.
    // (see https://www.rssboard.org/media-rss#media-license)

    extensions.join("\n")
}

pub fn media_rss(
    build_time: DateTime<Utc>,
    context: &Context,
    description: &Option<String>,
    site: &Site,
    site_path: &SitePath,
    title: &str,
    videos: &[Arc<Video>]
) {
    let feed_url = site.absolute_url_file_unchecked(MEDIA_RSS_FILENAME, site_path);

    let channel_extensions = "";

    let extra_namespaces = &[r#"xmlns:media="http://search.yahoo.com/mrss/""#];

    let xml = rss(
        build_time,
        channel_extensions,
        description,
        extra_namespaces,
        &feed_url,
        &mut item_extensions,
        site,
        title,
        videos
    );

    let path = context.build_dir.join(site_path.filesystem_relative()).join(MEDIA_RSS_FILENAME);
    fs::write(path, xml).unwrap();
}

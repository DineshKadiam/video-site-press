// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Atom specification for reference:
//! - https://validator.w3.org/feed/docs/atom.html (more approachable)
//! - https://datatracker.ietf.org/doc/html/rfc4287 (more formal)

use std::fs;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use indoc::formatdoc;

use hyper8_version::{VERSION_WITH_PATCH, VERSION_WITH_PATCH_AND_REVISION};

use crate::{
    Context,
    MarkdownSubset,
    Site,
    SitePath,
    Video
};
use crate::util::html_escape_outside_attribute;

use super::ATOM_FILENAME;

pub fn atom(
    build_time: DateTime<Utc>,
    context: &Context,
    description: &Option<String>,
    site: &Site,
    site_path: &SitePath,
    title: &str,
    videos: &[Arc<Video>]
) {
    let atom_feed_url = site.absolute_url_file_unchecked(ATOM_FILENAME, site_path);

    // According to the atom spec, <author> must be specified at least once in
    // the feed hierarchy for each entry (either globally on the feed or
    // directly on an entry or source). We approximate this as best as we can
    // by using the collection or playlist title as the feed author.
    let author_name = html_escape_outside_attribute(title);

    // TODO: The atom standard specifies atom:updated to be an "instant in
    // time when an entry or feed was modified in a way the publisher
    // considers significant", however right now we only know about the build
    // time, and we don't really know when significant changes happened
    // (our best approximation would be using the most recent date found on
    // any collection, playlist or video, but that would be highly
    // non-comprehensive). Some form of statefulness could change this
    // (for instance) - this would also tie in with similar questions around
    // RSS feeds - e.g. the "published" fields there. (however in general we
    // of course would prefer to avoid statefulness, so this should be
    // carefully weighed off)
    let build_time_rfc3339 = build_time.to_rfc3339();

    let entries = videos
        .iter()
        .map(|video| {
            entry(
                build_time,
                site,
                video
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // TODO: icon (optional field where we could use a hyper8 favicon)
    // "Identifies a small image which provides iconic visual identification for the feed. Icons should be square."
    // <icon>/icon.jpg</icon>
    // (see https://validator.w3.org/feed/docs/atom.html#optionalFeedElements)

    let site_url = site.absolute_url_unchecked();

    let subtitle = if let Some(description) = description {
        let description_html = MarkdownSubset::to_html(description);
        let description_escaped = html_escape_outside_attribute(&description_html);
        format!(r#"<subtitle type="html">{description_escaped}</subtitle>"#)
    } else {
        String::new()
    };

    let title_escaped = html_escape_outside_attribute(title);

    let xml = formatdoc!(r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
            <author>
                <name>{author_name}</name>
            </author>
            <generator uri="https://hyper8.org" version="{VERSION_WITH_PATCH}">
                Hyper 8 Video System {VERSION_WITH_PATCH_AND_REVISION}
            </generator>
            <id>{site_url}</id>
            <link href="{atom_feed_url}" rel="self"/>
            {subtitle}
            <title>{title_escaped}</title>
            <updated>{build_time_rfc3339}</updated>
            {entries}
        </feed>
    "#);

    let path = context.build_dir.join(site_path.filesystem_relative()).join(ATOM_FILENAME);
    fs::write(path, xml).unwrap();
}

fn entry(
    build_time: DateTime<Utc>,
    site: &Site,
    video: &Video
) -> String {
    // TODO: Same as with the updated field on the feed itself - but yet more
    // critical here - we don't know the published date of videos
    // (published as in "first appearance on the site/feed" or "something
    // fundamental about the video was changed so it should be marked
    // as 'updated on xxx'" rather than "first published anywhere
    // (else potentially)"), and this again would require some form of
    // statefulness (if we don't rely on the site operator adding their own
    // published dates, which we probably shouldn't).
    let build_time_rfc3339 = build_time.to_rfc3339();

    // TODO: published (optional field which could either use the video `date` or a stateful/user-supplied publishing date we don't have yet)
    // "Contains the time of the initial creation or first availability of the entry."
    // <published>2003-12-13T09:17:51-08:00</published>
    // (see https://validator.w3.org/feed/docs/atom.html#optionalEntryElements)

    let video_url = site.absolute_url_index_unchecked(&video.site_path);

    let summary = if let Some(description) = &video.description {
        let description_html = MarkdownSubset::to_html(description);
        let description_html_escaped = html_escape_outside_attribute(&description_html);
        format!(r#"<summary type="html">{description_html_escaped}</summary>"#)
    } else {
        String::new()
    };

    let title = video.title_or_slug_or_generic_label(&site.language.translations);
    let title_escaped = html_escape_outside_attribute(title);

    // TODO: Link can include things like language, type, enclosure (!), etc.,
    // we maybe should make use of that
    // (see https://validator.w3.org/feed/docs/atom.html#link)

    // TODO: Consider further fields to include in the future
    // See https://validator.w3.org/feed/docs/atom.html#recommendedEntryElements

    formatdoc!(r#"
        <entry>
            <id>{video_url}</id>
            <link href="{video_url}" rel="alternate"/>
            <title>{title_escaped}</title>
            {summary}
            <updated>{build_time_rfc3339}</updated>
        </entry>
    "#)
}

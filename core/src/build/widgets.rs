// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use indoc::formatdoc;

use hyper8_translations::Translations;
use hyper8_util::Attributes;
use hyper8_util::{casual_duration, html_escape_outside_attribute};

use crate::{
    Banner,
    Collection,
    Link,
    Site,
    Video
};

pub fn banner_html(banner: &Banner, prefix: &str) -> String {
    let mut attributes = Attributes::new();

    if let Some(description) = &banner.description {
        attributes.add_escaped("alt", description);
    }

    attributes.add_raw("class", "banner");

    if let Some(poster_assets) = &banner.assets {
        // TODO: Adjust up_to_xxx (depends on final content width we're using)
        let image_attributes = poster_assets.img_attributes_up_to_1280("banner", prefix);

        attributes.add_raw("src", image_attributes.src);
        attributes.add_raw("srcset", image_attributes.srcset);
    } else {
        attributes.add_raw("src", format!("{prefix}banner.jpg"));
    }

    // TODO: height="..." and width="..." so the browser can layout with the
    // intrinsic size before the image has loaded

    format!("<img {attributes}>")
}

/// Renders a list of "bubbles" (circles with a mesh gradient type background) that are
/// utilized in the layout to indicate the videos in a container (collection or playlist)
pub fn bubbles(prefix: &str, videos: &[Arc<Video>]) -> String {
    let bubbles = videos
        .iter()
        .map(|video| {
            if let Some(poster_meta) = &video.poster.as_ref().and_then(|poster| poster.poster_meta.as_ref()) {
                let background = poster_meta.lqip.background_css();
                let video_slug = video.slug_unchecked();

                formatdoc!(r#"
                    <li>
                        <a href="{prefix}{video_slug}/" style="{background}" tabindex="-1"></a>
                    </li>
                "#)
            } else {
                String::new()
            }
        })
        .collect::<Vec<String>>()
        .join("");

    format!(r#"<ul aria-hidden="true" class="bubbles">{bubbles}</ul>"#)
}

pub fn collection_context(
    collection: &Collection,
    site: &Site
) -> String {
    let index_suffix_file_only = site.index_suffix_file_only();
    let prefix = "../";

    let language = &site.language;
    let translations = &language.translations;

    let label = collection.title_or_slug_or_generic_label(translations);
    let label_escaped = html_escape_outside_attribute(label);

    let videos = collection.public_videos_in_collection_video_order(language);
    let video_bubbles = bubbles(prefix, &videos);

    format!(r#"
        <div class="parent_context" style="margin-top: 1rem;">
            <a href="{prefix}{index_suffix_file_only}">{label_escaped}</a>
            {video_bubbles}
        </div>
    "#)
}

/// A button enriched with data attributes that client scripting can use to
/// copy the content (embed code or link) to clipboard and display
/// success/failure state.
pub fn copy_button(
    content_key: &str,
    content_value: &str,
    label: &str
) -> String {
    let copy_icon = hyper8_icons::copy(None);
    format!(r#"
        <button data-{content_key}="{content_value}" data-copy>
            <span class="icon">{copy_icon}</span>
            <span>{label}</label>
        </button>
    "#)
}

/// <template> tags to be used in client-side (browser) scripting at runtime,
/// these are injected to the end of the body in the regular page and embed
/// layouts.
pub fn copy_button_icon_templates(translations: &Translations) -> String {
    let copy_icon = hyper8_icons::copy(None);
    let failed_icon = hyper8_icons::failure(translations.failed);
    let success_icon = hyper8_icons::success(translations.copied);

    formatdoc!(r#"
        <template data-copy-icon>{copy_icon}</template>
        <template data-failed-icon>{failed_icon}</template>
        <template data-success-icon>{success_icon}</template>
    "#)
}

pub fn link_button(link: &Link, translations: &Translations) -> String {
    let mut attributes = Attributes::new();

    attributes.add_raw("href", &link.url);

    if link.rel_me {
        attributes.add_raw("rel", "me");
    }

    let content = if link.hidden {
        attributes.add_raw("style", "display: none;");

        String::from("hidden")
    } else {
        attributes.add_raw("target", "_blank");

        let label = link.pretty_label();
        let link_icon = hyper8_icons::link(Some(translations.external_link));
        let escaped_label = html_escape_outside_attribute(&label);

        format!("{link_icon} <span>{escaped_label}</span>")
    };

    format!("<a {attributes}>{content}</a>")
}

/// Assumes that subscribe_permalink is a subfolder of the current location,
/// i.e. we render it from the perspective of /foo/bar/ and the subscribe
/// page is assumed to be at /foo/bar/[subscribe_permalink]/.
pub fn subscribe_button(
    subscribe_permalink: &str,
    site: &Site
) -> String {
    let index_suffix = site.index_suffix();
    let feed_icon = hyper8_icons::rss(Some(site.language.translations.feed));
    let t_subscribe = site.language.translations.subscribe;

    format!(r#"<a href="{subscribe_permalink}{index_suffix}">{feed_icon} {t_subscribe}</a>"#)
}

pub fn thumbnail(
    translations: &Translations,
    video: &Video,
    video_prefix: &str,
    with_duration: bool
) -> String {
    if let Some(poster) = &video.poster {
        let mut attributes = Attributes::new();

        // alt
        if let Some(description) = &poster.description {
            attributes.add_escaped("alt", description);
        }

        // class
        attributes.add_raw("class", "thumbnail");

        // loading
        attributes.add_raw("loading", "lazy");

        // sizes, src, srcset

        if let Some(poster_assets) = &poster.assets {
            let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);

            attributes.add_raw("sizes", image_attributes.sizes);
            attributes.add_raw("src", image_attributes.src);
            attributes.add_raw("srcset", image_attributes.srcset);
        } else {
            attributes.add_raw("src", format!("{video_prefix}poster.jpg"));
        }

        // height, style, width

        if let Some(poster_meta) = &poster.poster_meta {
            attributes.add_raw("height", poster_meta.height.to_string());
            attributes.add_raw("style", poster_meta.lqip.background_css());
            attributes.add_raw("width", poster_meta.width.to_string());
        }

        let img = format!("<img {attributes}>");

        if with_duration {
            let duration = match video.duration_public() {
                Some(duration) => {
                    let duration_formatted = casual_duration(duration, translations);
                    format!(r#"<span class="duration">{duration_formatted}</span>"#)
                }
                None => String::new()
            };

            format!(r#"<div class="thumbnail_wrapper">{img}{duration}</div>"#)
        } else {
            img
        }
    } else {
        String::new()
    }
}

pub fn unlisted_badge(translations: &Translations) -> String {
    let t_not_public = translations.not_public;
    format!(r#"<span class="unlisted_badge">{t_not_public}</span>"#)
}

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
    Language,
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

pub fn links(language: &Language, links: &[Link]) -> String {
    if links.is_empty() {
        return String::new();
    }

    let link_icon = hyper8_icons::link(Some(language.translations.external_link));

    let items = links
        .iter()
        .map(|link| {
            let rel_me = if link.rel_me { r#"rel="me""# } else { "" };
            let url = &link.url;

            if link.hidden {
                format!(r#"<a href="{url}" {rel_me} style="display: none;">hidden</a>"#)
            } else {
                let label = link.pretty_label();
                let e_label = html_escape_outside_attribute(&label);
                formatdoc!(r#"
                    <a href="{url}" {rel_me} target="_blank">{link_icon} <span>{e_label}</span></a>
                "#)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"<div class="links">{items}</div>"#)
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

    formatdoc!(r#"
        <div class="actions flex">
            <a href="{subscribe_permalink}{index_suffix}">
                {feed_icon} {t_subscribe}
            </a>
        </div>
    "#)
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

        // src, srcset

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

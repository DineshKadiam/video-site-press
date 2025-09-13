// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use indoc::formatdoc;

use hyper8_translations::Translations;

use crate::{
    Banner,
    Language,
    Link,
    Site,
    Video
};
use crate::util::{
    casual_duration,
    html_escape_outside_attribute
};

pub fn banner_html(banner: &Banner, prefix: &str) -> String {
    let alt = if let Some(description) = &banner.description {
        format!(r#"alt="{description}""#)
    } else {
        String::new()
    };

    if let Some(poster_assets) = &banner.assets {
        // TODO: Adjust up_to_xxx (depends on final content width we're using)
        let image_attributes = poster_assets.img_attributes_up_to_1280("banner", prefix);
        let src = image_attributes.src;
        let srcset = image_attributes.srcset;

        formatdoc!(r#"<img {alt} class="banner" src="{src}" srcset="{srcset}">"#)
    } else {
        formatdoc!(r#"<img {alt} class="banner" src="{prefix}banner.jpg">"#)
    }
}

/// Renders a list of "bubbles" (circles with a mesh gradient type background) that are
/// utilized in the layout to indicate the videos in a container (collection or playlist)
pub fn bubbles(prefix: &str, videos: &[Arc<Video>]) -> String {
    let bubbles = videos
        .iter()
        .map(|video| {
            if let Some(poster_meta) = &video.poster.as_ref().and_then(|poster| poster.poster_meta.as_ref()) {
                let background = poster_meta.lqip.gradient_radial();
                let video_slug = video.slug_unchecked();
                formatdoc!(r#"<li><a href="{prefix}{video_slug}/" style="{background}"></a></li>"#)
            } else {
                String::new()
            }
        })
        .collect::<Vec<String>>()
        .join("");

    format!(r#"<ul class="bubbles">{bubbles}</ul>"#)
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
    let failure_icon = hyper8_icons::failure(translations.failed);
    let success_icon = hyper8_icons::success(translations.copied);

    formatdoc!(r#"
        <template id="copy_icon">
            {copy_icon}
        </template>
        <template id="failure_icon">
            {failure_icon}
        </template>
        <template id="success_icon">
            {success_icon}
        </template>
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
    match &video.poster {
        Some(poster) => {
            let alt = if let Some(description) = &poster.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            let lqip_style = match &poster.poster_meta {
                Some(poster_meta) => {
                    let background = poster_meta.lqip.gradient_radial();
                    formatdoc!(r#"style="{background}""#)
                }
                None => String::new()
            };

            let img = match &poster.assets {
                Some(poster_assets) => {
                    let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);
                    let src = image_attributes.src;
                    let srcset = image_attributes.srcset;

                    format!(r#"<img {alt} class="thumbnail" src="{src}" srcset="{srcset}" {lqip_style}>"#)
                }
                None => {
                    format!(r#"<img {alt} class="thumbnail" src="{video_prefix}poster.jpg" {lqip_style}>"#)
                }
            };

            if !with_duration {
                return img;
            }

            let duration = match &video
                .highest_quality_video_file()
                .and_then(|video_file| video_file.video_meta.as_ref()) {
                Some(video_meta) => {
                    let duration = casual_duration(video_meta.duration, translations);
                    formatdoc!(r#"<span class="duration">{duration}</span>"#)
                }
                None => String::new()
            };

            formatdoc!(r#"
                <div class="thumbnail_wrapper">
                    {img}
                    {duration}
                </div>
            "#)
        },
        None => String::new()
    }
}

pub fn unlisted_badge(translations: &Translations) -> String {
    let t_not_public = translations.not_public;
    format!(r#"<span class="unlisted_badge">{t_not_public}</span>"#)
}

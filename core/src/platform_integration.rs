// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Various abstractions for integrating hyper8 with platforms (e.g. mastodon
//! and commercial walled gardens) by way of Open Graph metadata for link
//! previews, automatic audio/video embeds in timelines, and so on.
//!
//! Reference documentation and specifications:
//! - https://ogp.me/
//! - https://developers.facebook.com/docs/sharing/webmasters/#video

use hyper8_translations::Translations;
use hyper8_util::html_escape_inside_attribute;

use crate::Site;

pub struct Embed {
    pub height: usize,
    pub mime_type: &'static str,
    /// Absolute url to the embed page
    pub player_url: String,
    /// Absolute url to a video file
    pub video_url: String,
    pub width: usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlatformIntegration {
    ContentEmbeds,
    Disabled,
    LinkPreviews
}

pub struct PlatformIntegrationImage {
    pub height: u32,
    pub url: String,
    pub width: u32
}

pub struct PlatformIntegrationMeta {
    description: Option<String>,
    embed: Option<Embed>,
    image: Option<PlatformIntegrationImage>,
    image_alt: Option<String>,
    title: String,
    url: String
}

impl PlatformIntegration {
    pub fn content_embeds(&self) -> bool {
        match self {
            PlatformIntegration::ContentEmbeds => true,
            PlatformIntegration::Disabled => false,
            PlatformIntegration::LinkPreviews => false
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            PlatformIntegration::ContentEmbeds => true,
            PlatformIntegration::Disabled => false,
            PlatformIntegration::LinkPreviews => true
        }
    }

    pub fn from_key(key: &str) -> Option<PlatformIntegration> {
        match key {
            "content_embeds" => Some(PlatformIntegration::ContentEmbeds),
            "disabled" => Some(PlatformIntegration::Disabled),
            "link_previews" => Some(PlatformIntegration::LinkPreviews),
            _ => None
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            PlatformIntegration::ContentEmbeds => "content_embeds",
            PlatformIntegration::Disabled => "disabled",
            PlatformIntegration::LinkPreviews => "link_previews"
        }
    }

    pub fn label(&self, translations: &Translations) -> &str {
        match self {
            PlatformIntegration::ContentEmbeds => translations.link_previews_and_content_embeds,
            PlatformIntegration::Disabled => translations.disabled,
            PlatformIntegration::LinkPreviews => translations.link_previews_only
        }
    }
}

impl PlatformIntegrationMeta {
    pub fn description(&mut self, description: &str) {
        self.description = Some(description.to_string());
    }

    pub fn embed(
        &mut self,
        height: usize,
        mime_type: &'static str,
        player_url: String,
        video_url: String,
        width: usize
    ) {
        let embed = Embed {
            height,
            mime_type,
            player_url,
            video_url,
            width
        };

        self.embed = Some(embed)
    }

    pub fn image(&mut self, image: PlatformIntegrationImage) {
        self.image = Some(image);
    }

    pub fn image_alt(&mut self, description: &str) {
        self.image_alt = Some(description.to_string());
    }

    pub fn new(title: String, url: String) -> PlatformIntegrationMeta {
        PlatformIntegrationMeta {
            description: None,
            embed: None,
            image: None,
            image_alt: None,
            title,
            url
        }
    }

    pub fn tags(&self, site: &Site) -> String {
        let title_escaped = html_escape_inside_attribute(&self.title);

        let mut tags = Vec::new();

        // Open Graph metadata (for link previews and content embeds)

        // TODO: Uncomment and put the new audio(-only) content to use here
        // if let Some(audio) = &self.audio {
        //     tags.push(format!(r#"<meta property="og:audio" content="{audio}"/>"#));
        // }

        if let Some(description) = &self.description {
            let description_escaped = html_escape_inside_attribute(description);
            tags.push(format!(r#"<meta property="og:description" content="{description_escaped}"/>"#));
        }

        if let Some(image) = &self.image {
            tags.push(format!(r#"<meta property="og:image" content="{}"/>"#, image.url));
            tags.push(format!(r#"<meta property="og:image:height" content="{}"/>"#, image.height));
            tags.push(format!(r#"<meta property="og:image:width" content="{}"/>"#, image.width));
        }

        if let Some(image_alt) = &self.image_alt {
            let image_alt_escaped = html_escape_inside_attribute(image_alt);
            tags.push(format!(r#"<meta property="og:image:alt" content="{image_alt_escaped}"/>"#));
        }

        tags.push(format!(r#"<meta property="og:locale" content="{}"/>"#, &site.language.code));

        let site_name_escaped = html_escape_inside_attribute(site.title());
        tags.push(format!(r#"<meta property="og:site_name" content="{site_name_escaped}"/>"#));

        tags.push(format!(r#"<meta property="og:title" content="{title_escaped}"/>"#));

        tags.push(String::from(r#"<meta property="og:type" content="website"/>"#));
        tags.push(format!(r#"<meta property="og:url" content="{}"/>"#, self.url));

        if let Some(embed) = &self.embed {
            tags.push(format!(r#"<meta property="og:video" content="{}"/>"#, embed.video_url));
            tags.push(format!(r#"<meta property="og:video:height" content="{}"/>"#, embed.height));

            if site.base_url.as_ref().unwrap().is_https() {
                tags.push(format!(r#"<meta property="og:video:secure_url" content="{}"/>"#, embed.video_url));
            }

            tags.push(format!(r#"<meta property="og:video:width" content="{}"/>"#, embed.width));
        }

        // Twitter card metadata (only for content embeds)

        if let Some(embed) = &self.embed {
            tags.push(String::from(r#"<meta name="twitter:card" content="player" />"#));

            if let Some(description) = &self.description {
                let description_escaped = html_escape_inside_attribute(description);
                tags.push(format!(r#"<meta property="twitter:description" content="{description_escaped}"/>"#));
            }

            if let Some(image) = &self.image {
                tags.push(format!(r#"<meta property="twitter:image" content="{}"/>"#, image.url));
            }

            tags.push(format!(r#"<meta property="twitter:player" content="{}"/>"#, embed.player_url));
            tags.push(format!(r#"<meta property="twitter:player:height" content="{}"/>"#, embed.height));
            tags.push(format!(r#"<meta property="twitter:player:stream" content="{}"/>"#, embed.video_url));
            tags.push(format!(r#"<meta property="twitter:player:stream:content_type" content="{}"/>"#, embed.mime_type));
            tags.push(format!(r#"<meta property="twitter:player:width" content="{}"/>"#, embed.width));

            tags.push(format!(r#"<meta name="twitter:title" content="{title_escaped}" />"#));
        }

        tags.join("\n")
    }
}

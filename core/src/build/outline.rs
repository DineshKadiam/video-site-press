// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_util::{
    casual_duration,
    html_escape_outside_attribute
};

use crate::{
    Collection,
    Playlist,
    Site,
    SitePath,
    Video
};
use crate::build::thumbnail;
use crate::build::widgets::bubbles;
use crate::util::localized_date_d_mmm_yyyy;

pub struct Breadcrumb<'a> {
    pub href: String,
    pub step: &'a TraversalStep<'a>
}

/// Represents a traversal through the site tree, i.e. the steps taken to get
/// from the root of the site to a leaf node such as a collection, playlist
/// or video at any arbitrary level in the tree. We need this to render
/// hierarchy representations such as breadcrumbs and the likes.
#[derive(Clone)]
pub struct Traversal<'a> {
    /// Steps we have peeked through, i.e. in a context where we render a page,
    /// we know that these steps still lie below/ahead of us, i.e. we need to
    /// descend to them from our current location.
    peeked_steps: Vec<TraversalStep<'a>>,
    // TODO: Aim towards making this private again in the near future (probably makes sense to encapsulate access somehow)
    /// Steps we have actually traversed, i.e. in a context where we render a page,
    /// we know that these steps lie behind/above us, i.e. we need to ascend to them
    /// from our current location.
    pub steps: Vec<TraversalStep<'a>>
}

#[derive(Clone)]
pub enum TraversalStep<'a> {
    Collection(&'a Collection),
    Playlist(&'a Playlist),
    Video(&'a Video),
    /// E.g. an embed-codes/ page below a video page
    Virtual {
        label: &'a str
    }
}

impl<'a> Traversal<'a> {
    /// Relative url to go "back up" the hierarchy up to the root index
    /// (page), following the "taken steps".
    pub fn index_asc(&self, site: &Site) -> String {
        let prefix = self.prefix_asc();

        match site.clean_urls {
            true => prefix,
            false => prefix + "index.html"
        }
    }

    /// Url path to the index (page) at the peeked traversal.
    pub fn index_desc(&self, site: &Site) -> String {
        let prefix = self.prefix_desc();

        match site.clean_urls {
            true => prefix,
            false => prefix + "index.html"
        }
    }

    pub fn new() -> Traversal<'a> {
        Traversal {
            peeked_steps: Vec::new(),
            steps: Vec::new()
        }
    }

    pub fn peek_collection(&self, collection: &'a Collection) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.peeked_steps.push(TraversalStep::Collection(collection));
        clone
    }

    pub fn peek_playlist(&self, playlist: &'a Playlist) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.peeked_steps.push(TraversalStep::Playlist(playlist));
        clone
    }

    pub fn peek_video(&self, video: &'a Video) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.peeked_steps.push(TraversalStep::Video(video));
        clone
    }

    pub fn peeked_parents(&self, site: &Site) -> Vec<Breadcrumb<'_>> {
        if self.peeked_steps.is_empty() {
            return Vec::new();
        }

        // We always expect a last visited step, because anything we render always
        // exists within the context of some page, therefore there needs to be one
        // step at the minimum.
        let last_visited_step = &self.steps[self.steps.len() - 1];

        let peeked_parents_len = self.peeked_steps.len() - 1;
        self.peeked_steps[..peeked_parents_len]
            .iter()
            .map(|peeked_step| {
                let prefix = peeked_step.site_path_unchecked().normalized()
                    .strip_prefix(last_visited_step.site_path_unchecked().normalized())
                    .unwrap()
                    .to_string();

                let href = if site.clean_urls {
                    prefix
                } else {
                    format!("{prefix}index.html")
                };

                Breadcrumb { href, step: peeked_step }
            })
            .collect()
    }

    /// Relative url to go "back up" the hierarchy up to the root,
    /// following the "taken steps".
    pub fn prefix_asc(&self) -> String {
        "../".repeat(self.steps.len() - 1)
    }

    /// Relative url prefix to go "down" to the leaf node (= page) we're
    /// peeking at, relative to where are now (rather than to the site
    /// root). Returns e.g. "", "foo/", "foo/bar/", etc.
    pub fn prefix_desc(&self) -> String {
        match self.peeked_steps.iter().last() {
            Some(last_peeked_step) => {
                // We always expect a last visited step, because anything we render always
                // exists within the context of some page, therefore there needs to be one
                // step at the minimum.
                let last_visited_step = &self.steps[self.steps.len() - 1];

                last_peeked_step.site_path_unchecked().normalized()
                    .strip_prefix(last_visited_step.site_path_unchecked().normalized())
                    .unwrap()
                    .to_string()
            }
            None => String::new()
        }
    }

    pub fn site_path(&self) -> &SitePath {
        self.steps
            .iter()
            .rev()
            .find_map(|step| step.site_path())
            .unwrap()
    }

    pub fn with_collection(&self, collection: &'a Collection) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.steps.push(TraversalStep::Collection(collection));
        clone
    }

    pub fn with_playlist(&self, playlist: &'a Playlist) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.steps.push(TraversalStep::Playlist(playlist));
        clone
    }

    pub fn with_video(&self, video: &'a Video) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.steps.push(TraversalStep::Video(video));
        clone
    }

    pub fn with_virtual(&self, label: &'a str) -> Traversal<'a> {
        let mut clone = self.clone();
        clone.steps.push(TraversalStep::Virtual { label });
        clone
    }
}

impl TraversalStep<'_> {
    pub fn breadcrumb(&self, href: &str, site: &Site) -> String {
        match self {
            TraversalStep::Collection(collection) => {
                let title = collection.title_or_slug_or_generic_label(&site.language.translations);
                let e_title = html_escape_outside_attribute(title);
                format!(r#"<a href="{href}">{e_title}</a>"#)
            }
            TraversalStep::Playlist(playlist) => {
                let title = playlist.title_or_slug_or_generic_label(&site.language.translations);
                let e_title = html_escape_outside_attribute(title);
                format!(r#"<a href="{href}">{e_title}</a>"#)
            }
            TraversalStep::Video(video) => {
                let title = video.title_or_slug_or_generic_label(&site.language.translations);
                let e_title = html_escape_outside_attribute(title);
                format!(r#"<a href="{href}">{e_title}</a>"#)
            }
            TraversalStep::Virtual { label } => {
                let e_label = html_escape_outside_attribute(label);
                format!(r#"<a href="{href}">{e_label}</a>"#)
            }
        }
    }

    pub fn crumb_playlist(
        playlist: &Playlist,
        site: &Site,
        video: Option<&Video>
    ) -> String {
        let language = &site.language;
        let translations = &language.translations;

        // let banner = if let Some(banner) = &playlist.banner {
        //     banner_html(banner, prefix)
        // } else if let Some(video) = &playlist.public_videos()
        //     .find(|video| video.poster.is_some()) {
        //     let video_prefix = format!("{prefix}{}/", video.slug_unchecked());
        //     poster_html(video.poster.as_ref().unwrap(), &video_prefix)
        // } else {
        //     String::from(r#"<div class="placeholder"></div>"#)
        // };

        if let Some(video_active) = video {
            let index_suffix = site.index_suffix();
            let videos = playlist.public_videos_in_playlist_order(language);

            let header = {
                let index_suffix_file_only = site.index_suffix_file_only();
                let label = playlist.title_or_slug_or_generic_label(translations);
                let label_escaped = html_escape_outside_attribute(label);
                let prefix = "../";

                let video_bubbles = bubbles(prefix, &videos);

                format!(r#"
                    <div class="parent_context">
                        <a href="{prefix}{index_suffix_file_only}">{label_escaped}</a>
                        {video_bubbles}
                    </div>
                "#)
            };

            let items = videos
                .iter()
                .zip(1..)
                .map(|(video_iterated, number)| {
                    let label = video_iterated.title_or_slug_or_generic_label(translations);
                    let slug = video_iterated.slug_unchecked();

                    let video_prefix = if video_iterated.site_path == video_active.site_path {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = thumbnail(translations, video_iterated, &video_prefix, false);

                    let current;
                    let href;
                    let number_context;
                    if video_iterated.site_path == video_active.site_path {
                        current = "current";
                        href = String::new();
                        let t_current_video_xxx_of_xxx = translations.current_video_xxx_of_xxx(number, videos.len());
                        number_context = format!(r#"<span class="number_context">{t_current_video_xxx_of_xxx}</span>"#);
                    } else {
                        current = "";
                        href = format!("../{slug}{index_suffix}");
                        number_context = String::new();
                    }

                    let mut details = Vec::new();

                    if let Some(release_date) = video_iterated.release_date {
                        let date = localized_date_d_mmm_yyyy(release_date, language.chrono_locale);
                        details.push(date);
                    }

                    if let Some(duration) = video_iterated.duration_public() {
                        let duration_formatted = casual_duration(duration, translations);
                        details.push(duration_formatted);
                    }

                    let details = details.join(", ");

                    formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            {thumbnail_rendered}
                            <div class="description">
                                {number_context}
                                <span class="label">{label}</span>
                                <span class="details">
                                    {details}
                                </span>
                            </div>
                        </a>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            format!(r#"<div class="playlist_context">{header}{items}</div>"#)
        } else {
            String::new()
        }
    }

    pub fn site_path(&self) -> Option<&SitePath> {
        match self {
            TraversalStep::Collection(collection) => Some(&collection.site_path),
            TraversalStep::Playlist(playlist) => Some(&playlist.site_path),
            TraversalStep::Video(video) => Some(&video.site_path),
            TraversalStep::Virtual { .. } => None
        }
    }

    /// Only call when we know the step is not virtual, otherwise will panic
    pub fn site_path_unchecked(&self) -> &SitePath {
        match self {
            TraversalStep::Collection(collection) => &collection.site_path,
            TraversalStep::Playlist(playlist) => &playlist.site_path,
            TraversalStep::Video(video) => &video.site_path,
            TraversalStep::Virtual { .. } => unimplemented!()
        }
    }
}

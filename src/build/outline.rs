// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::{Datelike, NaiveDate};
use indoc::formatdoc;

use crate::{
    Collection,
    Desc,
    Playlist,
    Site,
    SitePath,
    Video
};
use crate::build::thumbnail;
use crate::build::widgets::{banner_html, bubbles, poster_html};
use crate::util::{
    casual_duration,
    html_escape_outside_attribute,
    localized_date_d_mmm_yyyy,
    localized_date_mmm_yyyy
};

pub struct Breadcrumb<'a> {
    pub href: String,
    pub step: &'a TraversalStep<'a>
}

pub enum CurrentPage<'a> {
    Collection(&'a Collection),
    Playlist(&'a Playlist),
    Video(&'a Video)
}

pub struct OutlineStats {
    pub earliest_release_date: Option<NaiveDate>,
    pub latest_release_date: Option<NaiveDate>,
    /// In seconds
    pub total_duration: f32,
    pub total_videos: usize
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

impl CurrentPage<'_> {
    pub fn is_collection(&self, collection: &Collection) -> bool {
        if let CurrentPage::Collection(contained_collection) = self {
            contained_collection.site_path == collection.site_path
        } else {
            false
        }
    }

    pub fn is_playlist(&self, playlist: &Playlist) -> bool {
        if let CurrentPage::Playlist(contained_playlist) = self {
            contained_playlist.site_path == playlist.site_path
        } else {
            false
        }
    }

    pub fn is_video(&self, video: &Video) -> bool {
        if let CurrentPage::Video(contained_video) = self {
            contained_video.site_path == video.site_path
        } else {
            false
        }
    }
}

impl OutlineStats {
    pub fn for_collection(collection: &Collection) -> OutlineStats {
        let mut outline_stats = OutlineStats {
            earliest_release_date: None,
            latest_release_date: None,
            total_duration: 0.0,
            total_videos: 0
        };

        outline_stats.register_collection(collection);

        outline_stats
    }

    pub fn for_playlist(playlist: &Playlist) -> OutlineStats {
        let mut outline_stats = OutlineStats {
            earliest_release_date: None,
            latest_release_date: None,
            total_duration: 0.0,
            total_videos: 0
        };

        outline_stats.register_playlist(playlist);

        outline_stats
    }

    fn register_collection(&mut self, collection: &Collection) {
        for playlist in collection.public_playlists() {
            self.register_playlist(playlist);
        }

        for subcollection in collection.public_subcollections() {
                self.register_collection(subcollection);
        }

        for video in collection.public_videos() {
            self.register_video(video);
        }
    }

    fn register_playlist(&mut self, playlist: &Playlist) {
        for video in playlist.public_videos() {
            self.register_video(video);
        }
    }

    fn register_release_date(&mut self, release_date_option: Option<NaiveDate>) {
        if let Some(release_date) = release_date_option {
            if self.earliest_release_date.is_none() ||
                self.earliest_release_date.is_some_and(|earliest_release_date| release_date < earliest_release_date) {
                self.earliest_release_date = Some(release_date);
            }
            if self.latest_release_date.is_none() ||
                self.latest_release_date.is_some_and(|latest_release_date| release_date > latest_release_date) {
                self.latest_release_date = Some(release_date);
            }
        }
    }

    fn register_video(&mut self, video: &Video) {
        self.register_release_date(video.release_date);

        if let Some(duration) = video.video_files
            .iter()
            .find_map(|video_file| {
                video_file.video_meta
                    .as_ref()
                    .map(|video_meta| video_meta.duration)
            }) {
            self.total_duration += duration;
        }

        self.total_videos += 1;
    }

    pub fn render(&self, site: &Site) -> String {
        if self.total_videos > 0 {
            let date_range = match (self.earliest_release_date, self.latest_release_date) {
                (Some(earliest_release_date), Some(latest_release_date)) => {
                    if earliest_release_date.month() != latest_release_date.month() ||
                        earliest_release_date.year() != latest_release_date.year() {
                        let earliest_formatted = localized_date_mmm_yyyy(earliest_release_date, site.language.chrono_locale);
                        let latest_formatted = localized_date_mmm_yyyy(latest_release_date, site.language.chrono_locale);
                        format!("{earliest_formatted} - {latest_formatted}, ")
                    } else {
                        let formatted = localized_date_mmm_yyyy(earliest_release_date, site.language.chrono_locale);
                        format!("{formatted}, ")
                    }
                }
                (Some(release_date), None) |
                (None, Some(release_date)) => {
                    let formatted = localized_date_mmm_yyyy(release_date, site.language.chrono_locale);
                    format!("{formatted}, ")
                }
                (None, None) => String::new()
            };

            let total_duration = casual_duration(self.total_duration, &site.language.translations);
            let total_videos = self.total_videos;
            let t_videos = &site.language.translations.videos;
            format!("{date_range}{total_videos} {t_videos}, {total_duration}")
        } else {
            let t_empty = &site.language.translations.empty;
            t_empty.to_string()
        }
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

    pub fn parents(&self, site: &Site) -> Vec<Breadcrumb<'_>> {
        let parents_len = self.steps.len() - 1;
        self.steps[..parents_len]
            .iter()
            .enumerate()
            .map(|(index, step)| {
                let levels_upwards = parents_len - index;
                let href = site.upwards_href(levels_upwards);
                Breadcrumb { href, step }
            })
            .collect()
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

    pub fn context_for_collection(
        &self,
        collection_viewed: &Collection,
        prefix: &str,
        site: &Site
    ) -> String {
        match self {
            TraversalStep::Collection(collection) => {
                let header = TraversalStep::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = TraversalStep::crumb_collection_context(collection, site, CurrentPage::Collection(collection_viewed));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            TraversalStep::Playlist(_) => unimplemented!(),
            TraversalStep::Video(_) => unimplemented!(),
            TraversalStep::Virtual { .. } => unimplemented!()
        }
    }

    pub fn context_for_playlist(
        &self,
        playlist: &Playlist,
        prefix: &str,
        site: &Site
    ) -> String {
        match self {
            TraversalStep::Collection(collection) => {
                let header = TraversalStep::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = TraversalStep::crumb_collection_context(collection, site, CurrentPage::Playlist(playlist));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            TraversalStep::Playlist(_) => unimplemented!(),
            TraversalStep::Video(_) => unimplemented!(),
            TraversalStep::Virtual { .. } => unimplemented!()
        }
    }

    pub fn context_for_video(
        &self,
        prefix: &str,
        site: &Site,
        video: &Video
    ) -> String {
        match self {
            TraversalStep::Collection(collection) => {
                let header = TraversalStep::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = TraversalStep::crumb_collection_context(collection, site, CurrentPage::Video(video));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            TraversalStep::Playlist(playlist) => TraversalStep::crumb_playlist(playlist, prefix, site, Some(video)),
            TraversalStep::Video(_) => unimplemented!(),
            TraversalStep::Virtual { .. } => unimplemented!()
        }
    }

    pub fn context_with_bubbles(
        &self,
        prefix: &str,
        site: &Site
    ) -> String {
        let index_suffix_file_only = site.index_suffix_file_only();
        let language = &site.language;
        let translations = &language.translations;

        match self {
            TraversalStep::Collection(collection) => {
                let label = collection.title_or_slug_or_generic_label(translations);
                let label_escaped = html_escape_outside_attribute(label);

                let videos = collection.public_videos_in_collection_video_order(language);
                let video_bubbles = bubbles(prefix, &videos);

                let inline = if videos.len() > 3 { "" } else { "inline" };

                format!(r#"
                    <div class="parent_context {inline}">
                        <a href="{prefix}{index_suffix_file_only}">{label_escaped}</a>
                        {video_bubbles}
                    </div>
                "#)
            }
            TraversalStep::Playlist(playlist) => {
                let label = playlist.title_or_slug_or_generic_label(translations);
                let label_escaped = html_escape_outside_attribute(label);

                let videos = playlist.public_videos_in_playlist_order(language);
                let video_bubbles = bubbles(prefix, &videos);

                let inline = if videos.len() > 3 { "" } else { "inline" };

                format!(r#"
                    <div class="parent_context {inline}">
                        <a href="{prefix}{index_suffix_file_only}">{label_escaped}</a>
                        {video_bubbles}
                    </div>
                "#)
            }
            TraversalStep::Video(_) => unimplemented!(),
            TraversalStep::Virtual { .. } => unimplemented!()
        }
    }

    fn crumb_collection_header(
        collection: &Collection,
        prefix: &str,
        site: &Site
    ) -> String {
        let index_suffix = site.index_suffix();

        let banner = if let Some(banner) = &collection.banner {
            banner_html(banner, prefix)
        } else if let Some(video) = &collection.public_videos().find(|video| video.poster.is_some()) { // TODO: Fall back to playlist/subcollection banner if no poster on video
            let video_prefix = format!("{prefix}{}/", video.slug_unchecked());
            poster_html(video.poster.as_ref().unwrap(), &video_prefix)
        } else {
            // TODO: Something graphically interesting instead (that also conveys "collection" somehow?)
            String::from(r#"<div class="placeholder"></div>"#)
        };
        let label = collection.title_or_slug_or_generic_label(&site.language.translations);
        let outline_stats = OutlineStats::for_collection(collection).render(site);

        formatdoc!(r#"
            <a href="{prefix}{index_suffix}">
                {banner}
                <div class="text">
                    <div class="medium_font">{label}</div>
                    <small>{outline_stats}</small>
                </div>
            </a>
        "#)
    }

    fn crumb_collection_context(
        collection: &Collection,
        site: &Site,
        current_page: CurrentPage
    ) -> String {
        let index_suffix = site.index_suffix();

        let language = &site.language;
        let translations = &language.translations;

        let subcollections = collection.public_subcollections_desc_by_release_date();

        let subcollections_context = if subcollections.is_empty() {
            String::new()
        } else {
            let rendered_subcollections = subcollections
                .iter()
                .map(|subcollection| {
                    let label = subcollection.title_or_slug_or_generic_label(translations);
                    let slug = subcollection.slug_unchecked();

                    let subcollection_prefix = if current_page.is_collection(collection) {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = if let Some(banner) = &subcollection.banner {
                        banner_html(banner, &subcollection_prefix)
                    } else if let Some(video) = subcollection.public_videos().find(|video| video.poster.is_some()) { // TODO: Take a banner from a present playlist or sub-subcollection too as alternative
                        let video_prefix = format!("{subcollection_prefix}{}/", video.slug_unchecked());
                        thumbnail(translations, video, &video_prefix, false)
                    } else {
                        String::new()
                    };

                    let (current, href) = if current_page.is_collection(subcollection) {
                        ("current", String::new())
                    } else {
                        ("", format!("../{slug}{index_suffix}"))
                    };

                    let outline_stats = OutlineStats::for_collection(subcollection).render(site);

                    formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            {thumbnail_rendered}
                            <div class="description">
                                <span class="label">{label}</span>
                                <span class="details">
                                    {outline_stats}
                                </span>
                            </div>
                        </a>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            formatdoc!(r#"
                <div class="playlist_context">
                    {rendered_subcollections}
                </div>
            "#)
        };

        let playlists = collection.public_playlists_desc_by_release_date();

        let playlists_context = if playlists.is_empty() {
            String::new()
        } else {
            let rendered_playlists = playlists
                .iter()
                .map(|playlist| {
                    let label = playlist.title_or_slug_or_generic_label(translations);
                    let slug = playlist.slug_unchecked();

                    let playlist_prefix = if current_page.is_playlist(playlist) {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = if let Some(banner) = &playlist.banner {
                        banner_html(banner, &playlist_prefix)
                    } else if let Some(video) = playlist
                        .public_videos()
                        .find(|video| video.poster.is_some()) {
                        let video_prefix = format!("{playlist_prefix}{}/", video.slug_unchecked());
                        thumbnail(translations, video, &video_prefix, false)
                    } else {
                        // TODO: Decide what condition this even is and handle
                        String::new()
                    };

                    let (current, href) = if current_page.is_playlist(playlist) {
                        ("current", String::new())
                    } else {
                        ("", format!("../{slug}{index_suffix}"))
                    };

                    let outline_stats = OutlineStats::for_playlist(playlist).render(site);

                    formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            {thumbnail_rendered}
                            <div class="description">
                                <span class="label">{label}</span>
                                <span class="details">
                                    {outline_stats}
                                </span>
                            </div>
                        </a>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            formatdoc!(r#"
                <div class="playlist_context">
                    {rendered_playlists}
                </div>
            "#)
        };

        let videos = collection.public_videos_sorted_by_release_date(Desc, language);

        let videos_context = if videos.is_empty() {
            String::new()
        } else {
            let rendered_videos = videos
                .iter()
                .map(|video| {
                    let label = video.title_or_slug_or_generic_label(translations);
                    let slug = video.slug_unchecked();

                    let video_prefix = if current_page.is_video(video) {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = thumbnail(translations, video, &video_prefix, false);

                    let (current, href) = if current_page.is_video(video) {
                        ("current", String::new())
                    } else {
                        ("", format!("../{slug}{index_suffix}"))
                    };

                    let duration = match &video
                        .highest_quality_video_file()
                        .and_then(|video_file| video_file.video_meta.as_ref()) {
                        Some(video_meta) => casual_duration(video_meta.duration, translations),
                        None => String::new()
                    };

                    formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            {thumbnail_rendered}
                            <div class="description">
                                <span class="label">{label}</span>
                                <span class="details">
                                    <span class="duration">{duration}</span>
                                </span>
                            </div>
                        </a>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("\n");

            formatdoc!(r#"
                <div class="playlist_context">
                    {rendered_videos}
                </div>
            "#)
        };

        formatdoc!(r#"
            {subcollections_context}
            {playlists_context}
            {videos_context}
        "#)
    }

    pub fn crumb_playlist(
        playlist: &Playlist,
        prefix: &str,
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

        if let Some(video) = video {
            let index_suffix = site.index_suffix();

            let videos = playlist.public_videos_in_playlist_order(language);

            let items = videos
                .iter()
                .enumerate()
                .map(|(index, video_iterated)| {
                    let label = video_iterated.title_or_slug_or_generic_label(translations);
                    let slug = video_iterated.slug_unchecked();

                    let video_prefix = if video_iterated.site_path == video.site_path {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = thumbnail(translations, video_iterated, &video_prefix, false);

                    let current;
                    let href;
                    let number_context;
                    if video_iterated.site_path == video.site_path {
                        current = "current";
                        href = String::new();
                        let count = videos.len();
                        let number = index + 1;
                        let playlist_label = playlist.title_or_slug_or_generic_label(translations);
                        let playlist_label_escaped = html_escape_outside_attribute(playlist_label);
                        let t_current_video_xxx_of_xxx_in_xxx = translations.current_video_xxx_of_xxx_in_xxx(number, &playlist_label_escaped, count);
                        number_context = format!(r#"<span class="number_context">{t_current_video_xxx_of_xxx_in_xxx}</span>"#);
                    } else {
                        current = "";
                        href = format!("../{slug}{index_suffix}");
                        number_context = String::new();
                    }

                    let mut details = Vec::new();

                    if let Some(release_date) = video.release_date {
                        let date = localized_date_d_mmm_yyyy(release_date, language.chrono_locale);
                        details.push(date);
                    }

                    if let Some(video_meta) = &video_iterated
                        .highest_quality_video_file()
                        .and_then(|video_file| video_file.video_meta.as_ref()) {
                        let duration = casual_duration(video_meta.duration, translations);
                        details.push(duration);
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

            formatdoc!(r#"
                <div class="playlist_context">
                    {items}
                </div>
            "#)
        } else {
            String::new()
        }
    }

    pub fn is_collection(&self, collection: &Collection) -> bool {
        if let TraversalStep::Collection(subcollection) = self {
            subcollection.site_path == collection.site_path
        } else {
            false
        }
    }

    pub fn is_playlist(&self) -> bool {
        if let TraversalStep::Playlist(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_video(&self, video: &Video) -> bool {
        if let TraversalStep::Video(contained_video) = self {
            contained_video.site_path == video.site_path
        } else {
            false
        }
    }

    /// Only call when we know the step is not virtual, otherwise will panic
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

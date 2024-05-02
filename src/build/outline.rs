use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use indoc::formatdoc;

use crate::{Collection, Context, Playlist, Site, Video};
use crate::build::playlist::banner_html;
use crate::build::thumbnail;
use crate::build::widgets::poster_html;
use crate::util::{casual_duration, localized_date_mmm_yyyy};

// TODO: Consider the similarity to Smartcrumb here (!) This indicates
//       we should really have a generic item of that sort for representing both?
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
pub enum Smartcrumb<'a> {
    Collection(&'a Collection),
    Playlist(&'a Playlist),
    Video(&'a Video)
}

impl<'a> CurrentPage<'a> {
    pub fn is_collection(&self, collection: &Collection) -> bool {
        if let CurrentPage::Collection(contained_collection) = self {
            contained_collection.path == collection.path
        } else {
            false
        }
    }

    pub fn is_playlist(&self, playlist: &Playlist) -> bool {
        if let CurrentPage::Playlist(contained_playlist) = self {
            contained_playlist.path == playlist.path
        } else {
            false
        }
    }

    pub fn is_video(&self, video: &Video) -> bool {
        if let CurrentPage::Video(contained_video) = self {
            contained_video.path == video.path
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
        for playlist in &collection.playlists {
            if !playlist.unlisted {
                self.register_playlist(playlist);
            }
        }

        for subcollection in &collection.subcollections {
            if !subcollection.unlisted {
                self.register_collection(subcollection);
            }
        }

        for video in &collection.videos {
            if !video.unlisted {
                self.register_video(video);
            }
        }
    }

    fn register_playlist(&mut self, playlist: &Playlist) {
        for video in &playlist.videos {
            if !video.unlisted {
                self.register_video(video);
            }
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

        if let Some(duration) = video.versions
            .iter()
            .find_map(|version| {
                version.video_meta
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

            let t_videos = &site.language.translations.videos;
            format!("{date_range}{} {t_videos}, {}", self.total_videos, casual_duration(self.total_duration))
        } else {
            let t_empty = &site.language.translations.empty;
            t_empty.to_string()
        }
    }
}

impl<'a> Smartcrumb<'a> {
    pub fn breadcrumb(&self, href: &str, site: &Site) -> String {
        match self {
            Smartcrumb::Collection(collection) => {
                let label = collection.label_build(site);
                format!(r#"<a href="{href}">{label}</a>"#)
            }
            Smartcrumb::Playlist(playlist) => {
                let label = playlist.label_build(site);
                format!(r#"<a href="{href}">{label}</a>"#)
            }
            Smartcrumb::Video(video) => {
                let label = video.label_build(site);
                format!(r#"<a href="{href}">{label}</a>"#)
            }
        }
    }

    pub fn context_for_collection(
        &self,
        collection_viewed: &Collection,
        context: &Context,
        prefix: &str,
        site: &Site
    ) -> String {
        match self {
            Smartcrumb::Collection(collection) => {
                let header = Smartcrumb::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = Smartcrumb::crumb_collection_context(collection, context, site, CurrentPage::Collection(collection_viewed));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            Smartcrumb::Playlist(_) => unimplemented!(),
            Smartcrumb::Video(_) => unimplemented!()
        }
    }

    pub fn context_for_playlist(
        &self,
        context: &Context,
        playlist: &Playlist,
        prefix: &str,
        site: &Site
    ) -> String {
        match self {
            Smartcrumb::Collection(collection) => {
                let header = Smartcrumb::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = Smartcrumb::crumb_collection_context(collection, context, site, CurrentPage::Playlist(playlist));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            Smartcrumb::Playlist(_) => unimplemented!(),
            Smartcrumb::Video(_) => unimplemented!()
        }
    }

    pub fn context_for_video(
        &self,
        context: &Context,
        prefix: &str,
        site: &Site,
        video: &Video
    ) -> String {
        match self {
            Smartcrumb::Collection(collection) => {
                let header = Smartcrumb::crumb_collection_header(collection, prefix, site);

                if prefix == "../" {
                    let list = Smartcrumb::crumb_collection_context(collection, context, site, CurrentPage::Video(video));
                    format!("{header}\n{list}")
                } else {
                    header
                }
            }
            Smartcrumb::Playlist(playlist) => Smartcrumb::crumb_playlist(context, playlist, prefix, site, Some(video)),
            Smartcrumb::Video(_) => unimplemented!()
        }
    }

    fn crumb_collection_header(
        collection: &Collection,
        prefix: &str,
        site: &Site
    ) -> String {
        // TODO: Fall back to playlist/subcollection banner if no poster on video
        let banner = if let Some(video) = &collection.videos
            .iter()
            .filter(|video| !video.unlisted)
            .find(|video| video.poster.is_some()) {
            let video_prefix = format!("{prefix}{}/", video.slug());
            poster_html(video.poster.as_ref().unwrap(), &video_prefix)
        } else {
            // TODO: Fall back to stock graphic thing that inhabits similar space as image
            String::new()
        };
        let label = collection.label_build(site);
        let outline_stats = OutlineStats::for_collection(collection).render(site);

        formatdoc!(r#"
            <a href="{prefix}">
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
        context: &Context,
        site: &Site,
        current_page: CurrentPage
    ) -> String {
        let index_suffix = context.index_suffix();

        let listed_subcollections: Vec<Arc<Collection>> = collection.subcollections
            .iter()
            .filter(|subcollection| !subcollection.unlisted)
            .map(|subcollection| Arc::clone(subcollection))
            .collect();

        let rendered_subcollections = listed_subcollections
            .iter()
            .map(|subcollection| {
                let label = subcollection.label_build(site);
                let slug = subcollection.slug();

                let subcollection_prefix = if current_page.is_collection(collection) {
                    String::new()
                } else {
                    format!("../{slug}/")
                };

                // TODO: Implement further, we could take a banner from this subcollection,
                // a banner from a playlist, a poster from a video, etc.
                let thumbnail_rendered = if let Some(video) = subcollection.videos
                    .iter()
                    .filter(|video| !video.unlisted)
                    .find(|video| video.poster.is_some()) {
                    let video_prefix = format!("{subcollection_prefix}{}/", video.slug());
                    thumbnail(video, &video_prefix, false)
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

        let subcollections_context = formatdoc!(r#"
            <div class="playlist_context">
                {rendered_subcollections}
            </div>
        "#);

        let listed_playlists: Vec<Arc<Playlist>> = collection.playlists
            .iter()
            .filter(|playlist| !playlist.unlisted)
            .map(|playlist| Arc::clone(playlist))
            .collect();

        let rendered_playlists = listed_playlists
            .iter()
            .map(|playlist| {
                let label = playlist.label_build(site);
                let slug = playlist.slug();

                let playlist_prefix = if current_page.is_playlist(playlist) {
                    String::new()
                } else {
                    format!("../{slug}/")
                };

                // TODO: playlist.listed_videos() helper method everywhere? (also listed_collections/playlists etc.)
                let thumbnail_rendered = if let Some(video) = playlist.videos
                    .iter()
                    .filter(|video| !video.unlisted)
                    .find(|video| video.poster.is_some()) {
                    let video_prefix = format!("{playlist_prefix}{}/", video.slug());
                    thumbnail(video, &video_prefix, false)
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

        let playlists_context = formatdoc!(r#"
            <div class="playlist_context">
                {rendered_playlists}
            </div>
        "#);

        let listed_videos: Vec<Arc<Video>> = collection.videos
            .iter()
            .filter(|video| !video.unlisted)
            .map(|video| Arc::clone(video))
            .collect();

        let rendered_videos = listed_videos
            .iter()
            .map(|video| {
                let label = video.label_build(site);
                let slug = video.slug();

                let video_prefix = if current_page.is_video(video) {
                    String::new()
                } else {
                    format!("../{slug}/")
                };

                let thumbnail_rendered = thumbnail(video, &video_prefix, false);

                let (current, href) = if current_page.is_video(video) {
                    ("current", String::new())
                } else {
                    ("", format!("../{slug}{index_suffix}"))
                };

                let duration = match &video.highest_quality_version().and_then(|version| version.video_meta.as_ref()) {
                    Some(video_meta) => casual_duration(video_meta.duration),
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

        let videos_context = formatdoc!(r#"
            <div class="playlist_context">
                {rendered_videos}
            </div>
        "#);

        formatdoc!(r#"
            {subcollections_context}
            {playlists_context}
            {videos_context}
        "#)
    }

    fn crumb_playlist(
        context: &Context,
        playlist: &Playlist,
        prefix: &str,
        site: &Site,
        video: Option<&Video>
    ) -> String {
        let banner = if let Some(banner) = &playlist.banner {
            banner_html(banner, prefix)
        } else if let Some(video) = &playlist.videos
            .iter()
            .filter(|video| !video.unlisted)
            .find(|video| video.poster.is_some()) {
            let video_prefix = format!("{prefix}{}/", video.slug());
            poster_html(video.poster.as_ref().unwrap(), &video_prefix)
        } else {
            // TODO: Fall back to stock graphic thing that inhabits similar space as image
            String::new()
        };
        let label = playlist.label_build(site);
        let outline_stats = OutlineStats::for_playlist(playlist).render(site);

        let header_crumb = formatdoc!(r#"
            <a href="{prefix}">
                {banner}
                <div class="text">
                    <div class="medium_font">{label}</div>
                    <small>{outline_stats}</small>
                </div>
            </a>
        "#);

        if let Some(video) = video {
            let index_suffix = context.index_suffix();

            let listed_videos: Vec<Arc<Video>> = playlist.videos
                .iter()
                .filter(|video| !video.unlisted)
                .map(|video| Arc::clone(video))
                .collect();

            let active_position = listed_videos
                .iter()
                .position(|video_iterated| video_iterated.path == video.path)
                .unwrap();

            let items = listed_videos
                .iter()
                .enumerate()
                .map(|(index, video_iterated)| {
                    let label = video_iterated.label_build(site);
                    let slug = video_iterated.slug();

                    let video_prefix = if index == active_position {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = thumbnail(video_iterated, &video_prefix, false);

                    let (current, href) = if index == active_position {
                        ("current", String::new())
                    } else {
                        ("", format!("../{slug}{index_suffix}"))
                    };

                    let duration = match &video_iterated.highest_quality_version().and_then(|version| version.video_meta.as_ref()) {
                        Some(video_meta) => casual_duration(video_meta.duration),
                        None => String::new()
                    };

                    let number = index + 1;
                    formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            <div class="number">{number}</div>
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

            let videos_crumb = formatdoc!(r#"
                <div class="playlist_context">
                    {items}
                </div>
            "#);

            formatdoc!(r#"
                {header_crumb}
                {videos_crumb}
            "#)
        } else {
            header_crumb
        }
    }

    pub fn is_collection(&self, collection: &Collection) -> bool {
        if let Smartcrumb::Collection(subcollection) = self {
            subcollection.path == collection.path
        } else {
            false
        }
    }

    pub fn is_playlist(&self) -> bool {
        if let Smartcrumb::Playlist(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_video(&self, video: &Video) -> bool {
        if let Smartcrumb::Video(contained_video) = self {
            contained_video.path == video.path
        } else {
            false
        }
    }
}

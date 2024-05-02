use chrono::{Datelike, NaiveDate};

use crate::{Collection, Playlist, Site, Video};
use crate::util::{casual_duration, localized_date_mmm_yyyy};

pub struct OutlineStats {
    pub earliest_release_date: Option<NaiveDate>,
    pub latest_release_date: Option<NaiveDate>,
    /// In seconds
    pub total_duration: f32,
    pub total_videos: usize
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

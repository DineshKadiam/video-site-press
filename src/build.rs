use std::fs;
use std::sync::Arc;

use indoc::formatdoc;
use rsubs_lib::srt;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::build::widgets::thumbnail;
use crate::util::{casual_duration, ensure_empty_dir, hard_link_or_copy};

mod collection;
mod outline;
mod playlist;
mod scripts;
mod styles;
pub mod translations;
mod video;
mod widgets;

use collection::collection_html;
use outline::OutlineStats;
use playlist::playlist_html;
use video::video_html;

#[derive(Clone)]
pub enum Smartcrumb<'a> {
    Collection(&'a Collection),
    Playlist(&'a Playlist),
    Video(&'a Video)
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

    // TODO: This only gives context in the form of a collection's videos, but completely
    //       omits subcollections and playlists in the collection.
    pub fn collection_context(&self, context: &Context, site: &Site, video: &Video) -> Option<String> {
        match self {
            Smartcrumb::Collection(collection) => {
                let index_suffix = context.index_suffix();

                let listed_videos: Vec<Arc<Video>> = collection.videos
                    .iter()
                    .filter(|video| !video.unlisted)
                    .map(|video| Arc::clone(video))
                    .collect();

                let items = listed_videos
                    .iter()
                    .map(|video_iterated| {
                        let label = video_iterated.label_build(site);
                        let slug = video_iterated.slug();

                        let video_prefix = if video_iterated.path == video.path {
                            String::new()
                        } else {
                            format!("../{slug}/")
                        };

                        let thumbnail_rendered = thumbnail(video_iterated, &video_prefix, false);

                        let (current, href) = if video_iterated.path == video.path {
                            ("current", String::new())
                        } else {
                            ("", format!("../{slug}{index_suffix}"))
                        };

                        let duration = match &video_iterated.highest_quality_version().and_then(|version| version.video_meta.as_ref()) {
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

                let label = collection.label_build(site);
                let outline_stats = OutlineStats::for_collection(collection).render(site);

                let header = formatdoc!(r#"
                    <div class="playlist_header">
                        <a class="medium_font title" href="../">{label}</a>
                        <small>{outline_stats}</small>
                    </div>
                "#);

                Some(
                    formatdoc!(r#"
                        <div class="playlist_context">
                            {header}
                            {items}
                        </div>
                    "#)
                )
            }
            _ => None
        }
    }

    pub fn is_video(&self, video: &Video) -> bool {
        if let Smartcrumb::Video(contained_video) = self {
            contained_video.path == video.path
        } else {
            false
        }
    }

    pub fn label(&self, href: &str, site: &Site) -> String {
        match self {
            Smartcrumb::Collection(collection) => {
                let label = collection.label_build(site);
                let outline_stats = OutlineStats::for_collection(collection).render(site);

                formatdoc!(r#"
                    <a href="{href}">
                        <div class="medium_font">{label}</div>
                        <small>{outline_stats}</small>
                    </a>
                "#)
            }
            Smartcrumb::Playlist(playlist) => {
                let label = playlist.label_build(site);
                let outline_stats = OutlineStats::for_playlist(playlist).render(site);

                formatdoc!(r#"
                    <a href="{href}">
                        <div class="medium_font">{label}</div>
                        <small>{outline_stats}</small>
                    </a>
                "#)
            }
            Smartcrumb::Video(_video) => unimplemented!()
        }
    }

    pub fn playlist_context(&self, context: &Context, site: &Site, video: &Video) -> Option<String> {
        match self {
            Smartcrumb::Playlist(playlist) => {
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

                let label = playlist.label_build(site);
                let outline_stats = OutlineStats::for_playlist(playlist).render(site);
                let header = formatdoc!(r#"
                    <div class="playlist_header">
                        <a class="medium_font title" href="../">{label}</a>
                        <small>{outline_stats}</small>
                    </div>
                "#);

                Some(
                    formatdoc!(r#"
                        <div class="playlist_context">
                            {header}
                            {items}
                        </div>
                    "#)
                )
            }
            _ => None
        }
    }
}

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    let outline = Vec::new();

    match &site.content {
        SiteContent::Collection(collection) => build_collection(context, collection, &site, &outline),
        SiteContent::Playlist(playlist) => build_playlist(context, playlist, &site, &outline),
        SiteContent::Video(video) => build_video(context, None, video, &site, &outline),
        SiteContent::Empty => (),
        SiteContent::Invalid => todo!()
    }

    scripts::generate(context, &site);
    styles::generate(context, &site);
}

fn build_collection(
    context: &Context,
    collection: &Collection,
    site: &Site,
    outline: &[Smartcrumb]
) {
    let mut outline_extended = outline.to_owned();
    outline_extended.push(Smartcrumb::Collection(collection));

    for subcollection in &collection.subcollections {
        build_collection(context, subcollection, site, &outline_extended);
    }

    for playlist in &collection.playlists {
        build_playlist(context, playlist, site, &outline_extended);
    }

    for video in &collection.videos {
        build_video(context, None, video, site, &outline_extended);
    }

    collection_html(context, collection, site, &outline_extended);
}

fn build_playlist(
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    outline: &[Smartcrumb]
) {
    let mut outline_extended = outline.to_owned();
    outline_extended.push(Smartcrumb::Playlist(playlist));

    if let Some(banner) = &playlist.banner {
        let playlist_build_dir = context.build_dir.join(&playlist.path);
        fs::create_dir_all(&playlist_build_dir).unwrap();

        if let Some(banner_assets) = &banner.assets {
            for asset in &banner_assets.all() {
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    playlist_build_dir.join(format!("banner_{}x{}.jpg", asset.width, asset.height))
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(&playlist.path).join(&banner.file_name),
                playlist_build_dir.join("banner.jpg")
            );
        }
    }

    playlist_html(context, playlist, site, &outline_extended);

    for video in &playlist.videos {
        build_video(context, Some(playlist), video, site, &outline_extended);
    }
}

fn build_video(
    context: &Context,
    playlist: Option<&Playlist>,
    video: &Video,
    site: &Site,
    outline: &[Smartcrumb]
) {
    let mut outline_extended = outline.to_owned();
    outline_extended.push(Smartcrumb::Video(video));

    let video_build_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_build_dir).unwrap();

    for version in &video.versions {
        hard_link_or_copy(
            &context.site_dir.join(&video.path).join(&version.file_name),
            video_build_dir.join(&version.file_name)
        );
    }

    if let Some(poster) = &video.poster {
        if let Some(poster_assets) = &poster.assets {
            for asset in &poster_assets.all() {
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    video_build_dir.join(format!("poster_{}x{}.jpg", asset.width, asset.height))
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(&video.path).join(&poster.file_name),
                video_build_dir.join("poster.jpg")
            );
        }
    }

    for (index, subtitle_file) in video.subtitles.iter().enumerate() {
        let source_path = &context.site_dir.join(&video.path).join(&subtitle_file.file_name);
        let target_path = video_build_dir.join(format!("{index}.vtt"));

        if subtitle_file.file_name.ends_with(".vtt") {
            hard_link_or_copy(source_path, target_path);
        } else if subtitle_file.file_name.ends_with(".srt") {
            // TODO: Conversion could/should be a (only if necessary:
            // background) job eventually to save on redundant processing and
            // wait time during build. Also possible errors should be
            // communicated as early as possible, e.g. checking the integrity
            // of the subtitle format right when we try to attach it to the video
            // on read_dir invocation.
            srt::parse(source_path.to_string_lossy().to_string())
                .unwrap()
                .to_vtt()
                .to_file(&target_path.to_string_lossy())
                .unwrap();
        }
    }

    video_html(context, playlist, video, site, &outline_extended);
}

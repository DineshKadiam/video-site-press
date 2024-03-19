use std::fs;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::util::{ensure_empty_dir, hard_link_or_copy};

mod collection;
mod playlist;
mod render;
mod styles;
mod video;

use collection::collection_html;
use playlist::playlist_html;
use video::video_html;

#[derive(Clone)]
pub enum Breadcrumb {
    Collection(String),
    Playlist(String),
    Video(String)
}

impl Breadcrumb {
    pub fn label(&self) -> &str {
        match self {
            Breadcrumb::Collection(label) => label,
            Breadcrumb::Playlist(label) => label,
            Breadcrumb::Video(label) => label
        }
    }
}

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    let breadcrumbs = Vec::new();

    match &site.content {
        SiteContent::Collection(collection) => build_collection(context, collection, &site, &breadcrumbs),
        SiteContent::Playlist(playlist) => build_playlist(context, playlist, &site, &breadcrumbs),
        SiteContent::Video(video) => build_video(context, video, &site, &breadcrumbs),
        SiteContent::Empty => (),
        SiteContent::Invalid => todo!()
    }

    fs::write(
        context.build_dir.join("scripts.js"),
        include_str!("build/assets/scripts.js")
    ).unwrap();

    styles::generate(context, &site);
}

fn build_collection(
    context: &Context,
    collection: &Collection,
    site: &Site,
    breadcrumbs: &[Breadcrumb]
) {
    let mut breadcrumbs_extended = breadcrumbs.to_owned();

    breadcrumbs_extended.push(Breadcrumb::Collection(collection.label().to_string()));

    for subcollection in &collection.subcollections {
        build_collection(context, subcollection, site, &breadcrumbs_extended);
    }

    for playlist in &collection.playlists {
        build_playlist(context, playlist, site, &breadcrumbs_extended);
    }

    for video in &collection.videos {
        build_video(context, video, site, &breadcrumbs_extended);
    }

    collection_html(context, collection, site, &breadcrumbs_extended);
}

fn build_playlist(
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    breadcrumbs: &[Breadcrumb]
) {
    let mut breadcrumbs_extended = breadcrumbs.to_owned();

    breadcrumbs_extended.push(Breadcrumb::Playlist(playlist.label().to_string()));

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

    playlist_html(context, playlist, site, &breadcrumbs_extended);

    for video in &playlist.videos {
        build_video(context, video, site, &breadcrumbs_extended);
    }
}

fn build_video(
    context: &Context,
    video: &Video,
    site: &Site,
    breadcrumbs: &[Breadcrumb]
) {
    let mut breadcrumbs_extended = breadcrumbs.to_owned();

    breadcrumbs_extended.push(Breadcrumb::Video(video.label().to_string()));

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

    video_html(context, video, site, &breadcrumbs_extended);
}

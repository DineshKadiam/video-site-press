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

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    match &site.content {
        SiteContent::Collection(collection) => build_collection(context, collection, &site),
        SiteContent::Playlist(playlist) => build_playlist(context, playlist, &site),
        SiteContent::Video(video) => build_video(context, video, &site),
        SiteContent::Empty => (),
        SiteContent::Invalid => todo!()
    }

    fs::write(
        context.build_dir.join("scripts.js"),
        include_str!("build/assets/scripts.js")
    ).unwrap();

    styles::generate(&context, &site);
}

fn build_collection(context: &Context, collection: &Collection, site: &Site) {
    for subcollection in &collection.subcollections {
        build_collection(context, subcollection, site);
    }

    for playlist in &collection.playlists {
        build_playlist(context, playlist, site);
    }

    for video in &collection.videos {
        build_video(context, video, site);
    }

    collection_html(context, collection, site);
}

fn build_playlist(context: &Context, playlist: &Playlist, site: &Site) {
    if let Some(poster) = &playlist.poster {
        let playlist_build_dir = context.build_dir.join(&playlist.path);
        fs::create_dir_all(&playlist_build_dir).unwrap();

        if let Some(poster_assets) = &poster.assets {
            for asset in &poster_assets.all() {
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    playlist_build_dir.join(format!("poster_{}x{}.jpg", asset.width, asset.height))
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(&playlist.path).join(&poster.file_name),
                playlist_build_dir.join("poster.jpg")
            );
        }
    }

    playlist_html(context, playlist, site);

    for video in &playlist.videos_physical {
        build_video(context, video, site);
    }
}

fn build_video(context: &Context, video: &Video, site: &Site) {
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

    video_html(context, video, site);
}

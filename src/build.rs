// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use rsubs_lib::SRT;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::build::widgets::thumbnail;
use crate::util::{ensure_empty_dir, hard_link_or_copy};

mod collection;
mod outline;
mod playlist;
mod scripts;
mod styles;
mod video;
mod widgets;

use collection::collection_html;
use outline::Traversal;
use playlist::playlist_html;
use video::video_html;

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    let traversal = Traversal::new();

    match &site.content {
        SiteContent::Collection(collection) => {
            if collection.offline { return; }
            build_collection(context, collection, &site, &traversal, false)
        }
        SiteContent::Playlist(playlist) => {
            if playlist.offline { return; }
            build_playlist(context, playlist, &site, &traversal, false)
        }
        SiteContent::Video(video) => {
            if video.offline { return; }
            build_video(context, None, video, &site, &traversal, false)
        }
        SiteContent::Empty => return,
        SiteContent::Invalid => todo!()
    }

    scripts::generate(context, &site);
    styles::generate(context, &site);
}

fn build_collection(
    context: &Context,
    collection: &Collection,
    site: &Site,
    traversal_previous: &Traversal,
    unlisted_inherited: bool
) {
    let traversal = traversal_previous.with_collection(collection);

    if let Some(banner) = &collection.banner {
        let collection_build_dir = context.build_dir.join(&collection.path);

        if &collection.path != "" {
            fs::create_dir_all(&collection_build_dir).unwrap();
        }

        if let Some(banner_assets) = &banner.assets {
            for asset in &banner_assets.all() {
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    collection_build_dir.join(format!("banner_{}x{}.jpg", asset.width, asset.height))
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(&collection.path).join(&banner.file_name),
                collection_build_dir.join("banner.jpg")
            );
        }
    }

    collection_html(context, collection, site, &traversal, unlisted_inherited);

    for subcollection in &collection.subcollections {
        if subcollection.offline { continue; }
        build_collection(context, subcollection, site, &traversal, unlisted_inherited || collection.unlisted);
    }

    for playlist in &collection.playlists {
        if playlist.offline { continue; }
        build_playlist(context, playlist, site, &traversal, unlisted_inherited || collection.unlisted);
    }

    for video in &collection.videos {
        if video.offline { continue; }
        build_video(context, None, video, site, &traversal, unlisted_inherited || collection.unlisted);
    }
}

fn build_playlist(
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    traversal_previous: &Traversal,
    unlisted_inherited: bool
) {
    let traversal = traversal_previous.with_playlist(playlist);

    if let Some(banner) = &playlist.banner {
        let playlist_build_dir = context.build_dir.join(&playlist.path);

        if &playlist.path != "" {
            fs::create_dir_all(&playlist_build_dir).unwrap();
        }

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

    playlist_html(context, playlist, site, &traversal, unlisted_inherited);

    for video in &playlist.videos {
        if video.offline { continue; }
        build_video(context, Some(playlist), video, site, &traversal, unlisted_inherited || playlist.unlisted);
    }
}

fn build_video(
    context: &Context,
    playlist: Option<&Playlist>,
    video: &Video,
    site: &Site,
    traversal_previous: &Traversal,
    unlisted_inherited: bool
) {
    let traversal = traversal_previous.with_video(video);

    let video_build_dir = context.build_dir.join(&video.path);

    if &video.path != "" {
        fs::create_dir_all(&video_build_dir).unwrap();
    }

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
            let srt_content = fs::read_to_string(source_path).unwrap();
            let vtt = SRT::parse(srt_content).unwrap().to_vtt();
            fs::write(target_path, vtt.to_string()).unwrap();
        }
    }

    video_html(context, playlist, video, site, &traversal, unlisted_inherited);
}

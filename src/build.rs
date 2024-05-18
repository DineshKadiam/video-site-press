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
use outline::Smartcrumb;
use playlist::playlist_html;
use video::video_html;

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
            let srt_content = fs::read_to_string(source_path).unwrap();
            let vtt = SRT::parse(srt_content).unwrap().to_vtt();
            fs::write(target_path, vtt.to_string()).unwrap();
        }
    }

    video_html(context, playlist, video, site, &outline_extended);
}

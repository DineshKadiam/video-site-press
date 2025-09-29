// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use hyper8_version::{VERSION_WITH_PATCH_AND_REVISION};
use rsubs_lib::SRT;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    ThemeKind,
    Video
};
use crate::build::widgets::thumbnail;
use crate::util::{ensure_empty_dir, hard_link_or_copy};

mod asset_hashes;
mod collection;
mod download;
mod embed;
mod embed_codes;
mod feeds;
mod layout;
mod outline;
mod player;
mod playlist;
mod scripts;
mod styles;
mod subscribe;
mod video;
mod widgets;

use collection::collection_html;
use download::download_html;
use embed::embed_html;
use embed_codes::embed_codes_html;
use layout::Layout;
use outline::Traversal;
use playlist::playlist_html;
use subscribe::subscribe_html;
use video::video_html;

use asset_hashes::AssetHashes;

const META_ROBOTS_NOINDEX_NOFOLLOW: &str = r#"<meta name="robots" content="noindex, nofollow">"#;

pub fn meta_generator() -> String {
    format!(r#"<meta name="generator" content="Hyper 8 Video System {VERSION_WITH_PATCH_AND_REVISION}">"#)
}

pub fn perform_build(context: &Context) {
    let mut site = context.get_site();

    while !site.all_metadata_computed() {
        println!("Waiting for banner/poster/video_meta computation");
        thread::sleep(Duration::from_secs(1));
        site = context.get_site();
    }

    ensure_empty_dir(&context.build_dir);

    let mut asset_hashes = AssetHashes::new();

    let traversal = Traversal::new();

    fs::write(
        context.build_dir.join("favicon.png"),
        include_bytes!("build/assets/favicon.png")
    ).unwrap();

    styles::generate_site_css(&mut asset_hashes, context, &site);
    scripts::generate_video_js(context);

    if let ThemeKind::Adaptive { .. } = site.theme.kind() {
        scripts::generate_adaptive_theme_js(context);
    }

    if site.copy_link_requested() || (site.embed_codes_requested() && site.base_url.is_some()) {
        scripts::generate_clipboard_js(context);
    }

    scripts::generate_navigation_js(&mut asset_hashes, context, &site);

    if site.base_url.is_some() {
        if site.embeds_requested() {
            scripts::generate_embed_js(context);
            styles::generate_embed_css(&mut asset_hashes, context, &site);
        }
    } else {
        if site.embeds_requested() {
            eprintln!("Embeds could not be generated because the site's base_url is not defined");
        }

        if site.feeds_requested() {
            eprintln!("Feeds could not be generated because the site's base_url is not defined");
        }

        if site.platform_integration_requested() {
            eprintln!("Metadata for platform integration could not be generated because the site's base_url is not defined");
        }
    }

    match &site.content {
        SiteContent::Collection(collection) => {
            if collection.offline() { return; }
            build_collection(
                &asset_hashes,
                context,
                collection,
                &site,
                &traversal
            );
        }
        SiteContent::Empty => (),
        SiteContent::Invalid { .. } => unimplemented!(),
        SiteContent::Playlist(playlist) => {
            if playlist.offline() { return; }
            build_playlist(
                &asset_hashes,
                context,
                playlist,
                &site,
                &traversal
            );
        }
        SiteContent::Video(video) => {
            if video.offline() { return; }
            build_video(
                &asset_hashes,
                context,
                None,
                &site,
                &traversal,
                video
            );
        }
    }
}

fn build_collection(
    asset_hashes: &AssetHashes,
    context: &Context,
    collection: &Collection,
    site: &Site,
    traversal_previous: &Traversal
) {
    let translations = &site.language.translations;
    let traversal = traversal_previous.with_collection(collection);

    let collection_build_dir = context.build_dir.join(collection.site_path.filesystem_relative());

    if !collection.site_path.is_root_path() {
        fs::create_dir_all(&collection_build_dir).unwrap();
    }

    if let Some(banner) = &collection.banner {
        if let Some(banner_assets) = &banner.assets {
            for asset in &banner_assets.all() {
                let target_filename = asset.target_filename("banner");
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    collection_build_dir.join(&target_filename)
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(collection.site_path.filesystem_relative()).join(&banner.file_name),
                collection_build_dir.join("banner.jpg")
            );
        }
    }

    let subscribe_permalink = if collection.feeds() && site.base_url.is_some() {
        let title = collection.title_or_slug_or_generic_label(translations);

        feeds::generate(
            context,
            &collection.description,
            site,
            &collection.site_path,
            title,
            &collection.public_videos_recursive()
        );

        let subscribe_permalink = collection.subscribe_permalink(translations);
        let traversal_subscribe = traversal.with_virtual(translations.subscribe);

        subscribe_html(
            asset_hashes,
            context,
            site,
            &collection.site_path,
            &subscribe_permalink,
            title,
            &traversal_subscribe,
            collection.unlisted()
        );

        Some(subscribe_permalink)
    } else {
        None
    };

    collection_html(
        asset_hashes,
        context,
        collection,
        site,
        subscribe_permalink,
        &traversal
    );

    for subcollection in &collection.subcollections {
        if subcollection.offline() { continue; }
        build_collection(
            asset_hashes,
            context,
            subcollection,
            site,
            &traversal
        );
    }

    for playlist in &collection.playlists {
        if playlist.offline() { continue; }
        build_playlist(
            asset_hashes,
            context,
            playlist,
            site,
            &traversal
        );
    }

    for video in &collection.videos {
        if video.offline() { continue; }
        build_video(
            asset_hashes,
            context,
            None,
            site,
            &traversal,
            video
        );
    }
}

fn build_playlist(
    asset_hashes: &AssetHashes,
    context: &Context,
    playlist: &Playlist,
    site: &Site,
    traversal_previous: &Traversal
) {
    let translations = &site.language.translations;
    let traversal = traversal_previous.with_playlist(playlist);

    let playlist_build_dir = context.build_dir.join(playlist.site_path.filesystem_relative());

    if !playlist.site_path.is_root_path() {
        fs::create_dir_all(&playlist_build_dir).unwrap();
    }

    if let Some(banner) = &playlist.banner {
        if let Some(banner_assets) = &banner.assets {
            for asset in &banner_assets.all() {
                let target_filename = asset.target_filename("banner");
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    playlist_build_dir.join(&target_filename)
                );
            }
        } else {
            hard_link_or_copy(
                context.site_dir.join(playlist.site_path.filesystem_relative()).join(&banner.file_name),
                playlist_build_dir.join("banner.jpg")
            );
        }
    }

    let subscribe_permalink = if playlist.feeds() && site.base_url.is_some() {
        let title = playlist.title_or_slug_or_generic_label(translations);

        feeds::generate(
            context,
            &playlist.description,
            site,
            &playlist.site_path,
            title,
            &playlist.public_videos().cloned().collect()
        );

        let subscribe_permalink = playlist.subscribe_permalink(translations);
        let traversal_subscribe = traversal.with_virtual(translations.subscribe);

        subscribe_html(
            asset_hashes,
            context,
            site,
            &playlist.site_path,
            &subscribe_permalink,
            title,
            &traversal_subscribe,
            playlist.unlisted()
        );

        Some(subscribe_permalink)
    } else {
        None
    };

    playlist_html(
        asset_hashes,
        context,
        playlist,
        site,
        subscribe_permalink,
        &traversal
    );

    for video in &playlist.videos {
        if video.offline() { continue; }
        build_video(
            asset_hashes,
            context,
            Some(playlist),
            site,
            &traversal,
            video
        );
    }
}

fn build_video(
    asset_hashes: &AssetHashes,
    context: &Context,
    playlist: Option<&Playlist>,
    site: &Site,
    traversal_previous: &Traversal,
    video: &Video
) {
    let translations = &site.language.translations;
    let traversal = traversal_previous.with_video(video);

    let video_build_dir = context.build_dir.join(video.site_path.filesystem_relative());

    if !video.site_path.is_root_path() {
        fs::create_dir_all(&video_build_dir).unwrap();
    }

    let default_streamable_audio_file = video.default_streamable_audio_file();

    if let Some(audio_file) = default_streamable_audio_file {
        hard_link_or_copy(
            context.site_dir.join(video.site_path.filesystem_relative()).join(&audio_file.file_name),
            video_build_dir.join(&audio_file.file_name)
        );
    }

    for video_file in video.streamable_video_files() {
        hard_link_or_copy(
            context.site_dir.join(video.site_path.filesystem_relative()).join(&video_file.file_name),
            video_build_dir.join(&video_file.file_name)
        );
    }

    if let Some(poster) = &video.poster {
        if let Some(poster_assets) = &poster.assets {
            for asset in &poster_assets.all() {
                let target_filename = asset.target_filename("poster");
                hard_link_or_copy(
                    context.cache_dir.join(&asset.file_name),
                    video_build_dir.join(&target_filename)
                );
            }
        }

        // TODO: Compute, cache and use a compressed/normalized (high-res, but
        // still constrained to a maximum resolution) version of the poster
        // for use here
        let original_extension = Path::new(&poster.file_name)
            .extension()
            .unwrap()
            .to_string_lossy();
        let original_filename = format!("poster.{original_extension}");

        hard_link_or_copy(
            context.site_dir.join(video.site_path.filesystem_relative()).join(&poster.file_name),
            video_build_dir.join(original_filename)
        );
    }

    for subtitle_file in video.valid_subtitles() {
        let id = &subtitle_file.id;
        let source_path = &context.site_dir
            .join(video.site_path.filesystem_relative())
            .join(&subtitle_file.file_name);

        // We always write a .vtt file for usage in the web player,
        // even if the source is a different format like .srt
        let target_path_vtt = video_build_dir.join(format!("{id}.vtt"));

        if subtitle_file.file_name.ends_with(".vtt") {
            fs::copy(source_path, target_path_vtt).unwrap();
        } else /* if subtitle_file.file_name.ends_with(".srt") */ {
            // If downloads are enabled for this video, also write the original
            // .srt file to the build so the original subtitle file can be
            // downloaded
            if video.download() {
                let target_path_srt = video_build_dir.join(format!("{id}.srt"));
                fs::copy(source_path, target_path_srt).unwrap();
            }

            // TODO: Conversion could/should be a (only if necessary:
            // background) job eventually to save on redundant processing and
            // wait time during build. Also possible errors should be
            // communicated as early as possible, e.g. checking the integrity
            // of the subtitle format right when we try to attach it to the video
            // on read_dir invocation.
            let srt_content = fs::read_to_string(source_path).unwrap();
            let vtt = SRT::parse(srt_content).unwrap().to_vtt();
            fs::write(target_path_vtt, vtt.to_string()).unwrap();
        }
    }

    if video.download() && video.has_downloadable_media() {
        // Write all downloadable audio files to build, excluding the default
        // streamable audio file, which we already have written above.
        for audio_file in video.downloadable_audio_files() {
            if default_streamable_audio_file.is_some_and(|streamable_audio_file| {
                streamable_audio_file == audio_file
            }) {
                continue;
            }

            hard_link_or_copy(
                context.site_dir.join(video.site_path.filesystem_relative()).join(&audio_file.file_name),
                video_build_dir.join(&audio_file.file_name)
            );
        }

        // Write all downloadable video files to build, excluding all
        // streamable video files, which we already have written above.
        for video_file in video.downloadable_video_files() {
            if video_file.streaming_restricted() {
                hard_link_or_copy(
                    context.site_dir.join(video.site_path.filesystem_relative()).join(&video_file.file_name),
                    video_build_dir.join(&video_file.file_name)
                );
            }
        }

        let traversal_download = traversal.with_virtual(translations.download);

        download_html(
            asset_hashes,
            context,
            site,
            video,
            &traversal_download
        );
    }

    if site.base_url.is_some() && video.has_streamable_media() {
        if video.embedding() {
            let traversal_embed_codes = traversal.with_virtual(translations.embed);

            embed_codes_html(
                asset_hashes,
                context,
                site,
                video,
                &traversal_embed_codes
            );
        }

        if video.embedding() || video.platform_integration().content_embeds() {
            let traversal_embed = traversal.with_virtual(translations.embed);

            embed_html(
                asset_hashes,
                context,
                playlist,
                site,
                video,
                &traversal_embed
            );
        }
    }

    video_html(
        asset_hashes,
        context,
        playlist,
        site,
        &traversal,
        video
    );
}

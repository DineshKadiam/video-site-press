// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

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

mod collection;
mod download;
mod embed;
mod embed_codes;
mod layout;
mod outline;
mod playlist;
mod scripts;
mod styles;
mod video;
mod widgets;

use collection::collection_html;
use download::download_html;
use embed::embed_html;
use embed_codes::embed_codes_html;
use layout::Layout;
use outline::Traversal;
use playlist::playlist_html;
use video::video_html;

/// Placed in the head of each page to identify the Hyper 8 build that generated it.
const META_GENERATOR: &str = concat!(r#"<meta name="generator" content="Hyper 8 Video System "#, env!("HYPER8_VERSION_DETAILED"), " (", env!("HYPER8_REVISION"), r#")">"#);
const META_ROBOTS_NOINDEX_NOFOLLOW: &str = r#"<meta name="robots" content="noindex, nofollow">"#;

/// When we link to assets on the rendered pages, we append a unique asset
/// hash to each path (e.g. "site.js?g1VVfPoEjUw"), which is derived from
/// the file content of the asset. We do this in order to prompt browsers to
/// fetch new, uncached assets when their content has changed. This struct
/// groups together those hashes for all assets we use.
pub struct AssetHashes {
    pub embed_css: Option<String>,
    pub site_css: Option<String>
}

impl AssetHashes {
    pub const ADAPTIVE_THEME_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_ADAPTIVE_THEME_JS");
    pub const CLIPBOARD_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_CLIPBOARD_JS");
    pub const EMBED_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_EMBED_JS");
    pub const VIDEO_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_VIDEO_JS");

    pub fn new() -> AssetHashes {
        AssetHashes {
            embed_css: None,
            site_css: None
        }
    }
}

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    let mut asset_hashes = AssetHashes::new();

    let traversal = Traversal::new();

    styles::generate_site_css(&mut asset_hashes, context, &site);
    scripts::generate_video_js(context);

    if let ThemeKind::Adaptive { .. } = site.theme.kind {
        scripts::generate_adaptive_theme_js(context);
    }

    let embeds_requested = site.embeds_requested();

    if embeds_requested {
        if site.base_url.is_some() {
            scripts::generate_embed_js(context);
            styles::generate_embed_css(&mut asset_hashes, context, &site);
        } else {
            // TODO: If we're using the graphical user interface this information needs to
            //       be communicated earlier/differently. E.g. maybe the embedding select
            //       should not even be accessible (= greyed out) with a note that the base_url
            //       needs to be set for this to do anything.
            //       Likewise for CLI usage it can be thought through further in which way this
            //       warning can be registered and in which context it can/should be printed.
            eprintln!("Embeds could not be generated because the site's base_url is not defined");
        }
    }

    if embeds_requested || site.copy_link_requested() {
        scripts::generate_clipboard_js(context);
    }

    if site.opengraph_requested() && site.base_url.is_none() {
        // TODO: If we're using the graphical user interface this information needs to
        //       be communicated earlier/differently. E.g. maybe the opengraph select
        //       should not even be accessible (= greyed out) with a note that the base_url
        //       needs to be set for this to do anything.
        //       Likewise for CLI usage it can be thought through further in which way this
        //       warning can be registered and in which context it can/should be printed.
        eprintln!("Open Graph metadata could not be generated because the site's base_url is not defined");
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
        SiteContent::Empty => return,
        SiteContent::Invalid => todo!()
    }
}

fn build_collection(
    asset_hashes: &AssetHashes,
    context: &Context,
    collection: &Collection,
    site: &Site,
    traversal_previous: &Traversal
) {
    let traversal = traversal_previous.with_collection(collection);

    if let Some(banner) = &collection.banner {
        let collection_build_dir = context.build_dir.join(&collection.path);

        if !collection.path.is_empty() {
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


    collection_html(
        asset_hashes,
        context,
        collection,
        site,
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
    let traversal = traversal_previous.with_playlist(playlist);

    if let Some(banner) = &playlist.banner {
        let playlist_build_dir = context.build_dir.join(&playlist.path);

        if !playlist.path.is_empty() {
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

    playlist_html(
        asset_hashes,
        context,
        playlist,
        site,
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

    let video_build_dir = context.build_dir.join(&video.path);

    if !video.path.is_empty() {
        fs::create_dir_all(&video_build_dir).unwrap();
    }

    for version in &video.versions {
        hard_link_or_copy(
            context.site_dir.join(&video.path).join(&version.file_name),
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

    if video.download() {
        let traversal_download = traversal.with_virtual(translations.download);

        download_html(
            asset_hashes,
            context,
            site,
            video,
            &traversal_download
        );
    }

    if video.embedding() {
        if let Some(base_url) = &site.base_url {
            let traversal_embed_codes = traversal.with_virtual(translations.embed);

            embed_codes_html(
                asset_hashes,
                base_url,
                context,
                site,
                video,
                &traversal_embed_codes
            );

            let traversal_embed = traversal.with_virtual(translations.embed);

            embed_html(
                asset_hashes,
                context,
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

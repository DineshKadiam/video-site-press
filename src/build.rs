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
    Video
};
use crate::build::widgets::thumbnail;
use crate::util::{ensure_empty_dir, hard_link_or_copy};

mod collection;
mod embed;
mod embed_codes;
mod outline;
mod playlist;
mod scripts;
mod styles;
mod video;
mod widgets;

use collection::collection_html;
use embed::embed_html;
use embed_codes::embed_codes_html;
use outline::Traversal;
use playlist::playlist_html;
use video::video_html;

/// Some options need to be evaluated in the context of the entire site tree
/// before they are applied at the individual node level. This includes
/// embedding, where the option is inherited down the tree but potentially
/// overriden at the node level, as well as the unlisted option, which
/// unconditionally applies to all descendants of a site tree branch whose
/// (in)direct parent node is marked as unlisted.
///
/// MergeOptions represents these options, and at the node level we only
/// consider the options on this struct (after we have merged in the options
/// from the node level beforehand of course).
#[derive(Clone)]
struct MergedOptions {
    pub embedding: bool,
    pub unlisted: bool
}

impl MergedOptions {
    pub fn default() -> MergedOptions {
        MergedOptions {
            embedding: false,
            unlisted: false
        }
    }

    pub fn merge_collection_options(&self, collection: &Collection) -> MergedOptions {
        let embedding = collection.embedding.unwrap_or(self.embedding);
        let unlisted = self.unlisted || collection.unlisted;

        MergedOptions {
            embedding,
            unlisted
        }
    }

    pub fn merge_playlist_options(&self, playlist: &Playlist) -> MergedOptions {
        let embedding = playlist.embedding.unwrap_or(self.embedding);
        let unlisted = self.unlisted || playlist.unlisted;

        MergedOptions {
            embedding,
            unlisted
        }
    }

    pub fn merge_video_options(&self, video: &Video) -> MergedOptions {
        let embedding = video.embedding.unwrap_or(self.embedding);
        let unlisted = self.unlisted || video.unlisted;

        MergedOptions {
            embedding,
            unlisted
        }
    }
}

pub fn perform_build(context: &Context) {
    let site = context.get_site();
    
    ensure_empty_dir(&context.build_dir);

    let default_options = MergedOptions::default();
    let traversal = Traversal::new();

    // When we traverse the entire site tree below, we flip this to true when
    // an embed is rendered anywhere. Later this tells us to add/write the
    // accompanying assets (css/js) for embeds to the build.
    let mut embeds_requested = false;

    match &site.content {
        SiteContent::Collection(collection) => {
            if collection.offline { return; }
            build_collection(
                context,
                collection,
                &mut embeds_requested,
                &default_options,
                &site,
                &traversal
            );
        }
        SiteContent::Playlist(playlist) => {
            if playlist.offline { return; }
            build_playlist(
                context,
                &mut embeds_requested,
                &default_options,
                playlist,
                &site,
                &traversal
            );
        }
        SiteContent::Video(video) => {
            if video.offline { return; }
            build_video(
                context,
                &mut embeds_requested,
                &default_options,
                None,
                &site,
                &traversal,
                video
            );
        }
        SiteContent::Empty => return,
        SiteContent::Invalid => todo!()
    }

    scripts::generate_site_js(context, &site);
    styles::generate_site_css(context, &site);

    if embeds_requested {
        if site.base_url.is_some() {
            scripts::generate_embed_js(context);
            styles::generate_embed_css(context, &site);
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
}

fn build_collection(
    context: &Context,
    collection: &Collection,
    embeds_requested: &mut bool,
    parent_options: &MergedOptions,
    site: &Site,
    traversal_previous: &Traversal
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

    let merged_options = parent_options.merge_collection_options(collection);

    collection_html(
        context,
        collection,
        &merged_options,
        site,
        &traversal
    );

    for subcollection in &collection.subcollections {
        if subcollection.offline { continue; }
        build_collection(
            context,
            subcollection,
            embeds_requested,
            &merged_options,
            site,
            &traversal
        );
    }

    for playlist in &collection.playlists {
        if playlist.offline { continue; }
        build_playlist(
            context,
            embeds_requested,
            &merged_options,
            playlist,
            site,
            &traversal
        );
    }

    for video in &collection.videos {
        if video.offline { continue; }
        build_video(
            context,
            embeds_requested,
            &merged_options,
            None,
            site,
            &traversal,
            video
        );
    }
}

fn build_playlist(
    context: &Context,
    embeds_requested: &mut bool,
    parent_options: &MergedOptions,
    playlist: &Playlist,
    site: &Site,
    traversal_previous: &Traversal
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

    let merged_options = parent_options.merge_playlist_options(playlist);

    playlist_html(
        context,
        &merged_options,
        playlist,
        site,
        &traversal
    );

    for video in &playlist.videos {
        if video.offline { continue; }
        build_video(
            context,
            embeds_requested,
            &merged_options,
            Some(playlist),
            site,
            &traversal,
            video
        );
    }
}

fn build_video(
    context: &Context,
    embeds_requested: &mut bool,
    parent_options: &MergedOptions,
    playlist: Option<&Playlist>,
    site: &Site,
    traversal_previous: &Traversal,
    video: &Video
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

    let merged_options = parent_options.merge_video_options(video);

    if merged_options.embedding {
        *embeds_requested = true;

        if let Some(base_url) = &site.base_url {
            let traversal_embed_codes = traversal.with_virtual("embed-codes", &site.language.translations.embed);

            embed_codes_html(
                base_url,
                context,
                &merged_options,
                site,
                video,
                &traversal_embed_codes
            );

            let traversal_embed = traversal.with_virtual("embed", &site.language.translations.embed);

            embed_html(
                context,
                site,
                video,
                &traversal_embed
            );
        }
    }

    video_html(
        context,
        &merged_options,
        playlist,
        site,
        &traversal,
        video
    );
}

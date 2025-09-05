// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use hyper8_translations::Translations;

use crate::{
    AudioFormat,
    AudioMetaCached,
    Collection,
    CollectionUpdate,
    Context,
    FileMeta,
    ImageProcessor,
    JobDesc,
    Playlist,
    PlaylistUpdate,
    PosterAsset,
    PosterAssets,
    PosterAssetsCached,
    PosterFile,
    PosterSourceCached,
    ResizeMode,
    Video,
    VideoFormat,
    VideoMetaCached,
    VideoUpdate
};
use crate::infallible_mutation;

use super::{encode, probe};

pub struct Worker {
    pub high_priority_only: bool,
    pub number: usize
}

impl Worker {
    pub fn compute_audio_metadata(
        context: &Context,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<Arc<Video>, String> {
        let mut audio_files_mut = video.audio_files.clone();

        let mut made_changes = false;

        for audio_file in audio_files_mut.iter_mut() {
            if audio_file.audio_meta.is_some() { continue }

            let audio_file_path = &context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&audio_file.file_name);

            match probe::probe_audio(audio_file_path, translations) {
                Ok(audio_meta) => {
                    audio_meta.write_cache_manifest(
                        context,
                        &audio_file.file_meta,
                        &audio_file.file_name,
                        &video.site_path
                    );

                    // Add/update video meta in in-memory cache
                    if let Ok(mut cache) = context.cache.lock() {
                        let version_path = format!("{}{}", &video.site_path.normalized(), audio_file.file_name);

                        if let Some(audio_meta_cached) = cache.audio_meta
                            .iter_mut()
                            .find(|cached| cached.path == version_path) {
                            audio_meta_cached.audio_meta = audio_meta.clone();
                            // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                            audio_meta_cached.file_meta = audio_file.file_meta.clone();
                        } else {
                            let audio_meta_cached = AudioMetaCached {
                                audio_meta: audio_meta.clone(),
                                file_meta: audio_file.file_meta.clone(),
                                path: version_path
                            };

                            cache.audio_meta.push(audio_meta_cached);
                        }
                    }

                    audio_file.audio_meta = Some(audio_meta);

                    made_changes = true;
                }
                Err(err) => {
                    let t_could_not_compute_metadata_for_xxx = translations.could_not_compute_metadata_for_xxx(audio_file_path);
                    let message = format!("{t_could_not_compute_metadata_for_xxx} ({err})");
                    return Err(message);
                }
            }
        }

        if made_changes {
            let mutation = |video_mut: &mut Video| {
                // TODO: It would be better to make granular updates, seeking just
                // the versions that we need to update and only updating those.
                // Right now we could discard newly added versions that weren't
                // present the moment we began looking for metadata.
                video_mut.audio_files = audio_files_mut;
                Ok(())
            };

            match context.update_video(mutation, &video.site_path) {
                VideoUpdate::Error(_) => infallible_mutation!(),
                VideoUpdate::InvalidManifest { .. } => {
                    let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                    Err(message)
                }
                VideoUpdate::NotFound => {
                    let message = translations.background_job_output_not_loaded_target_moved.to_string();
                    Err(message)
                }
                VideoUpdate::Success { video_updated, .. } => {
                    Ok(video_updated)
                }
            }
        } else {
            Ok(video)
        }
    }

    pub fn compute_collection_banner_assets(
        collection: Arc<Collection>,
        context: &Context,
        translations: &Translations
    ) -> Result<(), String> {
        if let Some(banner) = &collection.banner {
            let source_path = context.site_dir
                .join(collection.site_path.filesystem_relative())
                .join(&banner.file_name);

            let source_processor = ImageProcessor::open(&source_path)?;

            let min_overshoot = 1.1;

            let cover_rectangle_160 = ResizeMode::CoverCroppedRectangle { max_width: 160, min_aspect: 2.39 };
            let max_160 = Worker::compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

            let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                let cover_rectangle_320 = ResizeMode::CoverCroppedRectangle { max_width: 320, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
            } else {
                None
            };

            let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                let cover_rectangle_480 = ResizeMode::CoverCroppedRectangle { max_width: 480, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
            } else {
                None
            };

            let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                let cover_rectangle_800 = ResizeMode::CoverCroppedRectangle { max_width: 800, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
            } else {
                None
            };

            let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                let cover_rectangle_1280 = ResizeMode::CoverCroppedRectangle { max_width: 1280, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
            } else {
                None
            };

            let banner_assets = PosterAssets {
                max_160,
                max_320,
                max_480,
                max_800,
                max_1280,
                // TODO: For banners there is no configurable aspect, we could/should model this
                //       more cleanly in the future by using separate types for banner/poster assets
                poster_aspect: 0.0
            };

            // TODO: This internally allocates a PosterAssetsCached which it then serializes,
            //       but this could actually be re-used below (if we were to merge the two somehow would be good?)
            banner_assets.write_cache_manifest(
                context,
                &banner.file_meta,
                &banner.file_name,
                &collection.site_path
            );

            // Add/update poster assets in in-memory cache
            // TODO: This looks vaguely in need of encapsulation
            if let Ok(mut cache) = context.cache.lock() {
                let banner_path = format!("{}{}", collection.site_path.normalized(), banner.file_name);

                if let Some(poster_assets_cache) = cache.poster_assets
                    .iter_mut()
                    .find(|cached| cached.path == banner_path) {
                    poster_assets_cache.assets = banner_assets.clone();
                    // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                    poster_assets_cache.file_meta = banner.file_meta.clone();
                } else {
                    let poster_assets_cache = PosterAssetsCached {
                        assets: banner_assets.clone(),
                        file_meta: banner.file_meta.clone(),
                        path: banner_path
                    };

                    cache.poster_assets.push(poster_assets_cache);
                }
            }

            let mutation = |collection_mut: &mut Collection| {
                // Note that we expect the banner to be Some(), virtually always.
                // If it's missing that is an edge case with (likely) user interaction.
                if let Some(banner) = &mut collection_mut.banner {
                    banner.assets = Some(banner_assets);
                }
            };

            match context.update_collection(mutation, &collection.site_path) {
                CollectionUpdate::InvalidManifest { .. } => {
                    let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                    Err(message)
                }
                CollectionUpdate::NotFound => {
                    let message = translations.background_job_output_not_loaded_target_moved.to_string();
                    Err(message)
                }
                CollectionUpdate::Updated { .. } => Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// For now just computing banner assets if needed
    pub fn compute_collection_fast_tasks(
        collection: Arc<Collection>,
        context: &Context,
        translations: &Translations
    ) -> Result<(), String> {
        if collection.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            Worker::compute_collection_banner_assets(collection, context, translations)
        } else {
            Ok(())
        }
    }

    pub fn compute_playlist_banner_assets(
        context: &Context,
        playlist: Arc<Playlist>,
        translations: &Translations
    ) -> Result<(), String> {
        if let Some(banner) = &playlist.banner {
            let source_path = context.site_dir
                .join(playlist.site_path.filesystem_relative())
                .join(&banner.file_name);

            let source_processor = ImageProcessor::open(&source_path)?;

            let min_overshoot = 1.1;

            let cover_rectangle_160 = ResizeMode::CoverCroppedRectangle { max_width: 160, min_aspect: 2.39 };
            let max_160 = Worker::compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

            let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                let cover_rectangle_320 = ResizeMode::CoverCroppedRectangle { max_width: 320, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
            } else {
                None
            };

            let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                let cover_rectangle_480 = ResizeMode::CoverCroppedRectangle { max_width: 480, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
            } else {
                None
            };

            let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                let cover_rectangle_800 = ResizeMode::CoverCroppedRectangle { max_width: 800, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
            } else {
                None
            };

            let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                let cover_rectangle_1280 = ResizeMode::CoverCroppedRectangle { max_width: 1280, min_aspect: 2.39 };
                Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
            } else {
                None
            };

            let banner_assets = PosterAssets {
                max_160,
                max_320,
                max_480,
                max_800,
                max_1280,
                // TODO: For banners there is no configurable aspect, we could/should model this
                //       more cleanly in the future by using separate types for banner/poster assets
                poster_aspect: 0.0
            };

            // TODO: This internally allocates a PosterAssetsCached which it then serializes,
            //       but this could actually be re-used below (if we were to merge the two somehow would be good?)
            banner_assets.write_cache_manifest(
                context,
                &banner.file_meta,
                &banner.file_name,
                &playlist.site_path
            );

            // Add/update poster assets in in-memory cache
            // TODO: This looks vaguely in need of encapsulation
            if let Ok(mut cache) = context.cache.lock() {
                let banner_path = format!("{}{}", playlist.site_path.normalized(), banner.file_name);

                if let Some(poster_assets_cache) = cache.poster_assets
                    .iter_mut()
                    .find(|cached| cached.path == banner_path) {
                    poster_assets_cache.assets = banner_assets.clone();
                    // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                    poster_assets_cache.file_meta = banner.file_meta.clone();
                } else {
                    let poster_assets_cache = PosterAssetsCached {
                        assets: banner_assets.clone(),
                        file_meta: banner.file_meta.clone(),
                        path: banner_path
                    };

                    cache.poster_assets.push(poster_assets_cache);
                }
            }

            let mutation = |playlist_mut: &mut Playlist| {
                // Note that we expect the banner to be Some(), virtually always.
                // If it's missing that is an edge case with (likely) user interaction.
                if let Some(banner) = &mut playlist_mut.banner {
                    banner.assets = Some(banner_assets);
                }
            };

            match context.update_playlist(mutation, &playlist.site_path) {
                PlaylistUpdate::InvalidManifest { .. } => {
                    let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                    Err(message)
                }
                PlaylistUpdate::NotFound => {
                    let message = translations.background_job_output_not_loaded_target_moved.to_string();
                    Err(message)
                }
                PlaylistUpdate::Updated { .. } => Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// For now just computing banner assets if needed
    pub fn compute_playlist_fast_tasks(
        context: &Context,
        playlist: Arc<Playlist>,
        translations: &Translations
    ) -> Result<(), String> {
        if playlist.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            Worker::compute_playlist_banner_assets(context, playlist, translations)
        } else {
            Ok(())
        }
    }

    pub fn compute_poster(
        context: &Context,
        seek_offset: f32,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<Arc<Video>, String> {
        match encode::compute_poster(context, seek_offset, translations, &video) {
            Ok(file_name) => {
                let file_path = context.site_dir
                    .join(video.site_path.filesystem_relative())
                    .join(&file_name);

                let file_meta = FileMeta::new(&file_path);
                let poster_file = PosterFile::new(None, None, file_meta, file_name, None);

                // TODO: Are we missing the following (?): Creation of missing poster assets for the new poster?

                let mutation = |video_mut: &mut Video| {
                    video_mut.poster = Some(poster_file);
                    Ok(())
                };

                match context.update_video(mutation, &video.site_path) {
                    VideoUpdate::Error(_) => infallible_mutation!(),
                    VideoUpdate::InvalidManifest { .. } => {
                        let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                        Err(message)
                    }
                    VideoUpdate::NotFound => {
                        let message = translations.background_job_output_not_loaded_target_moved.to_string();
                        Err(message)
                    }
                    VideoUpdate::Success { video_updated, .. } => {
                        video_updated.write_manifest(context);
                        Ok(video_updated)
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub fn compute_poster_asset(
        context: &Context,
        source_image: &ImageProcessor,
        resize_mode: ResizeMode
    ) -> Result<PosterAsset, String> {
        let (file_name, dimensions) = source_image.resize(context, resize_mode);

        let filesize_bytes = match fs::metadata(context.cache_dir.join(&file_name)) {
            Ok(metadata) => metadata.len(),
            Err(err) => return Err(err.to_string())
        };

        Ok(
            PosterAsset {
                file_name,
                filesize_bytes,
                height: dimensions.1,
                width: dimensions.0
            }
        )
    }

    pub fn compute_poster_assets(
        context: &Context,
        poster_aspect: f32,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<(), String> {
        if let Some(poster) = &video.poster {
            let source_path = context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&poster.file_name);

            match ImageProcessor::open(&source_path) {
                Ok(source_processor) => {
                    let min_overshoot = 1.1;

                    let cover_rectangle_160 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 160 };
                    let max_160 = Worker::compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

                    let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                        let cover_rectangle_320 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 320 };
                        Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
                    } else {
                        None
                    };

                    let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                        let cover_rectangle_480 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 480 };
                        Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
                    } else {
                        None
                    };

                    let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                        let cover_rectangle_800 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 800 };
                        Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
                    } else {
                        None
                    };

                    let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                        let cover_rectangle_1280 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 1280 };
                        Some(Worker::compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
                    } else {
                        None
                    };

                    let poster_assets = PosterAssets {
                        max_160,
                        max_320,
                        max_480,
                        max_800,
                        max_1280,
                        poster_aspect
                    };

                    poster_assets.write_cache_manifest(
                        context,
                        &poster.file_meta,
                        &poster.file_name,
                        &video.site_path
                    );

                    // Add/update poster assets in in-memory cache
                    // TODO: This looks vaguely in need of encapsulation
                    if let Ok(mut cache) = context.cache.lock() {
                        let poster_path = format!("{}{}", video.site_path.normalized(), poster.file_name);

                        if let Some(poster_assets_cache) = cache.poster_assets
                            .iter_mut()
                            .find(|cached| cached.path == poster_path) {
                            poster_assets_cache.assets = poster_assets.clone();
                            // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                            poster_assets_cache.file_meta = poster.file_meta.clone();

                        } else {
                            let poster_assets_cache = PosterAssetsCached {
                                assets: poster_assets.clone(),
                                file_meta: poster.file_meta.clone(),
                                path: poster_path
                            };

                            cache.poster_assets.push(poster_assets_cache);
                        }
                    }

                    let mutation = |video: &mut Video| {
                        // Note that we expect the poster to be Some(), virtually always.
                        // If it's missing that is an edge case with (likely) user interaction.
                        if let Some(poster) = &mut video.poster {
                            poster.assets = Some(poster_assets);
                            Ok(())
                        } else {
                            let message = translations.background_job_output_not_loaded_target_removed.to_string();
                            Err(message)
                        }
                    };

                    match context.update_video(mutation, &video.site_path) {
                        VideoUpdate::Error(err) => Err(err),
                        VideoUpdate::InvalidManifest { .. } => {
                            let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                            Err(message)
                        }
                        VideoUpdate::NotFound => {
                            let message = translations.background_job_output_not_loaded_target_moved.to_string();
                            Err(message)
                        }
                        VideoUpdate::Success { .. } => Ok(())
                    }
                }
                Err(err) => Err(err)
            }
        } else {
            Ok(())
        }
    }

    pub fn compute_poster_meta(
        context: &Context,
        translations: &Translations,
        video: &Arc<Video>
    ) -> Result<(), String> {
        if let Some(poster) = &video.poster {
            let source_path = context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&poster.file_name);

            match ImageProcessor::open(&source_path) {
                Ok(source_processor) => {
                    let poster_meta = source_processor.compute_poster_meta();

                    poster_meta.write_cache_manifest(
                        context,
                        &poster.file_meta,
                        &poster.file_name,
                        &video.site_path
                    );

                    // Add/update poster source in in-memory cache
                    // TODO: This looks vaguely in need of encapsulation
                    if let Ok(mut cache) = context.cache.lock() {
                        let poster_path = format!("{}{}", video.site_path.normalized(), poster.file_name);

                        if let Some(poster_source_cached) = cache.poster_sources
                            .iter_mut()
                            .find(|cached| cached.path == poster_path) {
                            poster_source_cached.poster_meta = poster_meta.clone();
                            // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                            poster_source_cached.file_meta = poster.file_meta.clone();

                        } else {
                            let poster_source_cached = PosterSourceCached {
                                file_meta: poster.file_meta.clone(),
                                path: poster_path,
                                poster_meta: poster_meta.clone(),
                            };

                            cache.poster_sources.push(poster_source_cached);
                        }
                    }

                    let mutation = |video: &mut Video| {
                        // Note that we expect the poster to be Some(), virtually always.
                        // If it's missing that is an edge case with (likely) user interaction.
                        if let Some(poster) = &mut video.poster {
                            poster.poster_meta = Some(poster_meta);
                            Ok(())
                        } else {
                            let message = translations.background_job_output_not_loaded_target_removed.to_string();
                            Err(message)
                        }
                    };

                    match context.update_video(mutation, &video.site_path) {
                        VideoUpdate::Error(err) => Err(err),
                        VideoUpdate::InvalidManifest { .. } => {
                            let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                            Err(message)
                        }
                        VideoUpdate::NotFound => {
                            let message = translations.background_job_output_not_loaded_target_moved.to_string();
                            Err(message)
                        }
                        VideoUpdate::Success { .. } => Ok(())
                    }
                }
                Err(err) => Err(err)
            }
        } else {
            Ok(())
        }
    }

    /// Includes computing video metadata for all video files, as well as poster,
    /// poster meta and poster assets computation
    pub fn compute_video_fast_tasks(
        context: &Context,
        poster_aspect: f32,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<(), String> {
        let video_with_metadata = Worker::compute_video_metadata(context, translations, video)?;

        if let Some(poster) = &video_with_metadata.poster {
            if poster.poster_meta.is_none() {
                Worker::compute_poster_meta(
                    context,
                    translations,
                    &video_with_metadata
                )?;
            }

            if poster.assets.is_none() {
                Worker::compute_poster_assets(
                    context,
                    poster_aspect,
                    translations,
                    video_with_metadata
                )?;
            }

            Ok(())
        } else {
            let video_with_poster = Worker::compute_poster(
                context,
                0.5,
                translations,
                video_with_metadata
            )?;

            Worker::compute_poster_meta(
                context,
                translations,
                &video_with_poster
            )?;

            Worker::compute_poster_assets(
                context,
                poster_aspect,
                translations,
                video_with_poster
            )?;

            Ok(())
        }
    }

    pub fn compute_video_metadata(
        context: &Context,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<Arc<Video>, String> {
        let mut video_files_mut = video.video_files.clone();

        let mut made_changes = false;

        for video_file in video_files_mut.iter_mut() {
            if video_file.video_meta.is_some() { continue }

            let video_file_path = context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&video_file.file_name);

            match probe::probe_video(&video_file_path, translations) {
                Ok(video_meta) => {
                    video_meta.write_cache_manifest(
                        context,
                        &video_file.file_meta,
                        &video_file.file_name,
                        &video.site_path
                    );

                    // Add/update video meta in in-memory cache
                    // TODO: This looks vaguely in need of encapsulation
                    if let Ok(mut cache) = context.cache.lock() {
                        let version_path = format!("{}{}", &video.site_path.normalized(), video_file.file_name);

                        if let Some(video_meta_cached) = cache.video_meta
                            .iter_mut()
                            .find(|cached| cached.path == version_path) {
                            // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                            video_meta_cached.file_meta = video_file.file_meta.clone();
                            video_meta_cached.video_meta = video_meta.clone();

                        } else {
                            let video_meta_cached = VideoMetaCached {
                                file_meta: video_file.file_meta.clone(),
                                path: version_path,
                                video_meta: video_meta.clone()
                            };

                            cache.video_meta.push(video_meta_cached);
                        }
                    }

                    video_file.video_meta = Some(video_meta);

                    made_changes = true;
                }
                Err(err) => {
                    let t_could_not_compute_metadata_for_xxx = translations.could_not_compute_metadata_for_xxx(&video_file_path);
                    let message = format!("{t_could_not_compute_metadata_for_xxx} ({err})");
                    return Err(message);
                }
            }
        }

        if made_changes {
            let mutation = |video_mut: &mut Video| {
                // TODO: It would be better to make granular updates, seeking just
                // the versions that we need to update and only updating those.
                // Right now we could discard newly added versions that weren't
                // present the moment we began looking for metadata.
                video_mut.video_files = video_files_mut;
                Ok(())
            };

            match context.update_video(mutation, &video.site_path) {
                VideoUpdate::Error(_) => infallible_mutation!(),
                VideoUpdate::InvalidManifest { .. } => {
                    let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                    Err(message)
                }
                VideoUpdate::NotFound => {
                    let message = translations.background_job_output_not_loaded_target_moved.to_string();
                    Err(message)
                }
                VideoUpdate::Success { video_updated, .. } => {
                    Ok(video_updated)
                }
            }
        } else {
            Ok(video)
        }
    }

    pub fn new(high_priority_only: bool, number: usize) -> Worker {
        Worker {
            high_priority_only,
            number
        }
    }

    pub fn pick_video_poster(
        context: &Context,
        poster_aspect: f32,
        timecode: f32,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<(), String> {
        let video_with_metadata = Worker::compute_video_metadata(context, translations, video)?;

        if let Some(highest_quality_video_file) = &video_with_metadata.highest_quality_video_file() {
            let video_meta = highest_quality_video_file.video_meta.as_ref().unwrap();

            let bounded_timecode = if timecode < 0.0 {
                0.0
            } else if timecode > video_meta.duration {
                video_meta.duration
            } else {
                timecode
            };

            let video_with_computed_poster = Worker::compute_poster(
                context,
                bounded_timecode,
                translations,
                video_with_metadata
            )?;

            Worker::compute_poster_assets(
                context,
                poster_aspect,
                translations,
                video_with_computed_poster
            )
        } else {
            let site_path = video_with_metadata.site_path.normalized();
            let t_there_was_no_source_video_to_compute_a_poster_from = translations.there_was_no_source_video_to_compute_a_poster_from;
            let message = format!("{t_there_was_no_source_video_to_compute_a_poster_from} ({site_path})");
            Err(message)
        }
    }

    pub fn render_audio_format(
        audio_format: AudioFormat,
        context: &Arc<Context>,
        job_id: String,
        translations: &Translations,
        video: Arc<Video>
    ) -> Result<(), String> {
        match encode::encode_audio(audio_format, context, job_id, translations, &video) {
            Ok(audio_file) => {
                let mutation = |video_mut: &mut Video| {
                    video_mut.audio_files.push(audio_file);
                    Ok(())
                };

                match context.update_video(mutation, &video.site_path) {
                    VideoUpdate::Error(_) => infallible_mutation!(),
                    VideoUpdate::InvalidManifest { .. } => {
                        let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                        Err(message)
                    }
                    VideoUpdate::NotFound => {
                        let message = translations.background_job_output_not_loaded_target_moved.to_string();
                        Err(message)
                    }
                    VideoUpdate::Success { video_updated, .. } => {
                        // TODO: Assign audio_meta ourselves? (in the mutation above already?), as we should have all
                        //       the details at hand anyway? (we know from which source we transcoded, which should have necessary meta)
                        let audio_meta_job_desc = JobDesc::AudioMeta {
                            video: Arc::clone(&video_updated)
                        };

                        context.queue_job(audio_meta_job_desc);

                        Ok(())
                    }
                }
            }
            Err(err) => {
                let t_encoding_a_new_audio_file_failed = translations.encoding_a_new_audio_file_failed;
                let message = format!("{t_encoding_a_new_audio_file_failed} ({err})");
                Err(message)
            }
        }
    }

    pub fn render_video_format(
        context: &Arc<Context>,
        job_id: String,
        translations: &Translations,
        video: Arc<Video>,
        video_format: VideoFormat
    ) -> Result<(), String> {
        match encode::encode_video(context, job_id, translations, &video, video_format) {
            Ok(video_file) => {
                let mutation = |video_mut: &mut Video| {
                    video_mut.video_files.push(video_file);
                    Ok(())
                };

                match context.update_video(mutation, &video.site_path) {
                    VideoUpdate::Error(_) => infallible_mutation!(),
                    VideoUpdate::InvalidManifest { .. } => {
                        let message = translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string();
                        Err(message)
                    }
                    VideoUpdate::NotFound => {
                        let message = translations.background_job_output_not_loaded_target_moved.to_string();
                        Err(message)
                    }
                    VideoUpdate::Success { video_updated, .. } => {
                        let site = context.get_site();
                        let video_fast_tasks_job_desc = JobDesc::VideoFastTasks {
                            poster_aspect: site.poster_aspect.numeric(),
                            video: Arc::clone(&video_updated)
                        };

                        context.queue_job(video_fast_tasks_job_desc);

                        Ok(())
                    }
                }
            }
            Err(err) => {
                let t_encoding_a_new_video_file_failed = translations.encoding_a_new_video_file_failed;
                let message = format!("{t_encoding_a_new_video_file_failed} ({err})");
                Err(message)
            }
        }
    }
}

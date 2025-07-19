// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    FileMeta,
    ImageProcessor,
    JobKind,
    Playlist,
    PlaylistUpdate,
    PosterAsset,
    PosterAssets,
    PosterAssetsCached,
    PosterFile,
    PosterSourceCached,
    ResizeMode,
    Video,
    VideoMetaCached,
    VideoUpdate
};
use crate::transcode;

fn compute_collection_banner_assets(
    context: &Context,
    collection: Arc<Collection>
) -> Result<(), String> {
    if let Some(banner) = &collection.banner {
        let source_path = context.site_dir
            .join(collection.site_path.filesystem_relative())
            .join(&banner.file_name);

        match ImageProcessor::open(&source_path) {
            Ok(source_processor) => {
                let min_overshoot = 1.1;

                let cover_rectangle_160 = ResizeMode::CoverCroppedRectangle { max_width: 160, min_aspect: 2.39 };
                let max_160 = compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

                let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                    let cover_rectangle_320 = ResizeMode::CoverCroppedRectangle { max_width: 320, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
                } else {
                    None
                };

                let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                    let cover_rectangle_480 = ResizeMode::CoverCroppedRectangle { max_width: 480, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
                } else {
                    None
                };

                let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                    let cover_rectangle_800 = ResizeMode::CoverCroppedRectangle { max_width: 800, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
                } else {
                    None
                };

                let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                    let cover_rectangle_1280 = ResizeMode::CoverCroppedRectangle { max_width: 1280, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
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
                    CollectionUpdate::NotFound => {
                        return Err(String::from("Banner assets could not be assigned in the site tree as the collection has meanwhile changed path."));
                    }
                    CollectionUpdate::PendingErrors { .. } => {
                        return Err(String::from("Banner assets could not be assigned in the site tree as the collection manifest currently has pending errors."));
                    }
                    CollectionUpdate::Updated { .. } => ()
                }
            }
            Err(err) => return Err(err)
        }
    }

    Ok(())
}

fn compute_metadata(context: &Context, video: Arc<Video>) -> Arc<Video> {
    let mut versions_mut = video.versions.clone();

    let mut made_changes = false;

    for version in versions_mut.iter_mut() {
        if version.video_meta.is_some() { continue }

        if let Ok(video_meta) = transcode::ffprobe(&context.site_dir.join(video.site_path.filesystem_relative()).join(&version.file_name)) {
            video_meta.write_cache_manifest(
                context,
                &version.file_meta,
                &version.file_name,
                &video.site_path
            );

            // Add/update video meta in in-memory cache
            // TODO: This looks vaguely in need of encapsulation
            if let Ok(mut cache) = context.cache.lock() {
                let version_path = format!("{}{}", &video.site_path.normalized(), version.file_name);

                if let Some(video_meta_cached) = cache.video_meta
                    .iter_mut()
                    .find(|cached| cached.path == version_path) {
                    // TODO: Not sure if updating the file_meta is necessary (should be identical?) - think through carefully.
                    video_meta_cached.file_meta = version.file_meta.clone();
                    video_meta_cached.video_meta = video_meta.clone();

                } else {
                    let video_meta_cached = VideoMetaCached {
                        file_meta: version.file_meta.clone(),
                        path: version_path,
                        video_meta: video_meta.clone()
                    };

                    cache.video_meta.push(video_meta_cached);
                }
            }

            version.video_meta = Some(video_meta);

            made_changes = true;
        }
    }

    if made_changes {
        let mutation = |video_mut: &mut Video| {
            // TODO: It would be better to make granular updates, seeking just
            // the versions that we need to update and only updating those.
            // Right now we could discard newly added versions that weren't
            // present the moment we began looking for metadata.
            video_mut.versions = versions_mut;
            Ok(())
        };

        match context.update_video(mutation, &video.site_path) {
            VideoUpdate::Error(_) => unreachable!("Infallible mutation"),
            VideoUpdate::NotFound => {
                eprintln!("Metadata for video {} could not be assigned in the site tree as the video has meanwhile changed path.", video.site_path.normalized());
            }
            VideoUpdate::PendingErrors { .. } => {
                eprintln!("Metadata for video {} could not be assigned in the site tree as the video currently has pending errors.", video.site_path.normalized());
            }
            VideoUpdate::Success { video_updated, .. } => {
                // TODO: Reactivate
                // context.create_version_jobs(&video);
                return video_updated;
            }
        }
    }

    video
}

fn compute_playlist_banner_assets(context: &Context, playlist: Arc<Playlist>) -> Result<(), String> {
    if let Some(banner) = &playlist.banner {
        let source_path = context.site_dir
            .join(playlist.site_path.filesystem_relative())
            .join(&banner.file_name);

        match ImageProcessor::open(&source_path) {
            Ok(source_processor) => {
                let min_overshoot = 1.1;

                let cover_rectangle_160 = ResizeMode::CoverCroppedRectangle { max_width: 160, min_aspect: 2.39 };
                let max_160 = compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

                let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                    let cover_rectangle_320 = ResizeMode::CoverCroppedRectangle { max_width: 320, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
                } else {
                    None
                };

                let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                    let cover_rectangle_480 = ResizeMode::CoverCroppedRectangle { max_width: 480, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
                } else {
                    None
                };

                let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                    let cover_rectangle_800 = ResizeMode::CoverCroppedRectangle { max_width: 800, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
                } else {
                    None
                };

                let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                    let cover_rectangle_1280 = ResizeMode::CoverCroppedRectangle { max_width: 1280, min_aspect: 2.39 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
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
                    PlaylistUpdate::NotFound => {
                        return Err(String::from("Banner assets could not be assigned in the site tree as the playlist has meanwhile changed path."));
                    }
                    PlaylistUpdate::PendingErrors { .. } => {
                        return Err(String::from("Banner assets could not be assigned in the site tree as the playlist has meanwhile changed path."));
                    }
                    PlaylistUpdate::Updated { .. } => ()
                }
            }
            Err(err) => return Err(err)
        }
    }

    Ok(())
}

fn compute_poster(context: &Context, seek_offset: f32, video: Arc<Video>) -> Arc<Video> {
    if let Ok(file_name) = transcode::compute_poster(context, seek_offset, &video) {
        let file_path = context.site_dir
            .join(video.site_path.filesystem_relative())
            .join(&file_name);

        let file_meta = FileMeta::new(&file_path);
        let poster_file = PosterFile::new(None, None, file_meta, file_name, None);

        let mutation = |video_mut: &mut Video| {
            video_mut.poster = Some(poster_file);
            Ok(())
        };

        match context.update_video(mutation, &video.site_path) {
            VideoUpdate::Error(_) => unreachable!("Infallible mutation"),
            VideoUpdate::NotFound => {
                eprintln!("Poster for video {} could not be assigned in the site tree as the video has meanwhile changed path.", video.site_path.normalized());
            }
            VideoUpdate::PendingErrors { .. } => {
                eprintln!("Poster for video {} could not be assigned in the site tree as there are currently errors in the video manifest.", video.site_path.normalized());
            }
            VideoUpdate::Success { video_updated, .. } => {
                video_updated.write_manifest(context);
                return video_updated;
            }
        }
    }

    video
}

fn compute_poster_asset(
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

fn compute_poster_assets(context: &Context, poster_aspect: f32, video: Arc<Video>) -> Result<(), String> {
    if let Some(poster) = &video.poster {
        let source_path = context.site_dir
            .join(video.site_path.filesystem_relative())
            .join(&poster.file_name);

        match ImageProcessor::open(&source_path) {
            Ok(source_processor) => {
                let min_overshoot = 1.1;

                let cover_rectangle_160 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 160 };
                let max_160 = compute_poster_asset(context, &source_processor, cover_rectangle_160)?;

                let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
                    let cover_rectangle_320 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 320 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_320)?)
                } else {
                    None
                };

                let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
                    let cover_rectangle_480 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 480 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_480)?)
                } else {
                    None
                };

                let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
                    let cover_rectangle_800 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 800 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_800)?)
                } else {
                    None
                };

                let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
                    let cover_rectangle_1280 = ResizeMode::CoverRectangle { aspect: poster_aspect, max_width: 1280 };
                    Some(compute_poster_asset(context, &source_processor, cover_rectangle_1280)?)
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
                        Err(String::from("Poster assets could not be assigned as the poster was meanwhile removed"))
                    }
                };

                match context.update_video(mutation, &video.site_path) {
                    VideoUpdate::Error(err) => {
                        return Err(err)
                    }
                    VideoUpdate::NotFound => {
                        return Err(String::from("Poster assets could not be assigned in the site tree as the video has meanwhile changed path."));
                    }
                    VideoUpdate::PendingErrors { .. } => {
                        return Err(String::from("Poster assets could not be assigned in the site tree as the video currently has pending errors."));
                    }
                    VideoUpdate::Success { .. } => ()
                }
            }
            Err(err) => return Err(err)
        }
    }

    Ok(())
}

fn compute_poster_meta(
    context: &Context,
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
                        Err(String::from("Poster meta could not be assigned as the poster was meanwhile removed"))
                    }
                };

                match context.update_video(mutation, &video.site_path) {
                    VideoUpdate::Error(err) => {
                        return Err(err)
                    }
                    VideoUpdate::NotFound => {
                        return Err(String::from("Poster meta could not be assigned in the site tree as the video has meanwhile changed path."));
                    }
                    VideoUpdate::PendingErrors { .. } => {
                        return Err(String::from("Poster meta could not be assigned in the site tree as the video currently has pending errors."));
                    }
                    VideoUpdate::Success { .. } => ()
                }
            }
            Err(err) => return Err(err)
        }
    }

    Ok(())
}

pub fn start_threads(count: usize, context_ref: &Arc<Context>) {
	for index in 0..count {
        let context = Arc::clone(context_ref);

        thread::spawn(move || {
            let worker_number = index + 1;
            let high_priority_only = worker_number == 1;

            loop {
                let job = context.take_processing_job(high_priority_only);

                if job.is_none() {
                    thread::sleep(Duration::from_millis(1000));
                    continue
                }

                let job = job.unwrap();

                match job.kind {
                    JobKind::CollectionFastTasks(collection) => {
                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &collection.site_path.normalized());
                        let process_id = context.processing.lock().unwrap().push(collection.site_path.normalized(), label);
                        if let Some(banner) = &collection.banner {
                            if banner.assets.is_none() {
                                if let Err(err) = compute_collection_banner_assets(&context, collection) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::PlaylistFastTasks(playlist) => {
                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &playlist.site_path.normalized());
                        let process_id = context.processing.lock().unwrap().push(playlist.site_path.normalized(), label);

                        if let Some(banner) = &playlist.banner {
                            if banner.assets.is_none() {
                                if let Err(err) = compute_playlist_banner_assets(&context, playlist) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoFastTasks { poster_aspect, video } => {
                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &video.site_path.normalized());
                        let process_id = context.processing
                            .lock()
                            .unwrap()
                            .push(video.site_path.normalized(), label);

                        let video_with_metadata = compute_metadata(&context, video);

                        if let Some(poster) = &video_with_metadata.poster {
                            if poster.poster_meta.is_none() {
                                if let Err(err) = compute_poster_meta(&context, &video_with_metadata) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }

                            if poster.assets.is_none() {
                                if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_metadata) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }
                        } else {
                            let video_with_poster = compute_poster(&context, 0.5, video_with_metadata);

                            if let Err(err) = compute_poster_meta(&context, &video_with_poster) {
                                context.processing_errors.lock().unwrap().push(err);
                            }

                            if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_poster) {
                                context.processing_errors.lock().unwrap().push(err);
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoFormat { format, video } => {
                        let format_label = format.label(); // Implement Display so we can just drop it right into interpolation?
                        let label = format!("[Worker {worker_number}] Computing version {format_label} for {}", video.site_path.normalized());
                        let process_id = context.processing.lock().unwrap().push(video.site_path.normalized(), label);

                        match transcode::encode(&context, format, process_id, &video) {
                            Ok(version) => {
                                let mutation = |video_mut: &mut Video| {
                                    video_mut.versions.push(version);
                                    Ok(())
                                };

                                match context.update_video(mutation, &video.site_path) {
                                    VideoUpdate::Error(_) => unreachable!("The mutation is infallible"),
                                    VideoUpdate::NotFound => {
                                        let err = String::from("Video could not be updated with newly computed version because it was not found at its site path anymore.");
                                        context.processing_errors.lock().unwrap().push(err);
                                    }
                                    VideoUpdate::PendingErrors { .. } => {
                                        let err = String::from("Video could not be updated with newly computed version as the video currently has pending errors.");
                                        context.processing_errors.lock().unwrap().push(err);
                                    }
                                    VideoUpdate::Success { video_updated, .. } => {
                                        let site = context.get_site();
                                        let job_kind = JobKind::VideoFastTasks {
                                            poster_aspect: site.poster_aspect.numeric(),
                                            video: Arc::clone(&video_updated)
                                        };
                                        context.push_job_queue(job_kind);
                                    }
                                }
                            }
                            Err(()) => {
                                context.processing_errors.lock().unwrap().push("Encoding a new version failed for an unknown reason.".to_string())
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoPickPoster { poster_aspect, timecode, video } => {
                        let label = format!("[Worker {worker_number}] Computing random poster for {}", &video.site_path.normalized());
                        let process_id = context.processing.lock().unwrap().push(video.site_path.normalized(), label);

                        let video_with_metadata = compute_metadata(&context, video);

                        if let Some(highest_quality_version) = &video_with_metadata.highest_quality_version() {
                            let video_meta = highest_quality_version.video_meta.as_ref().unwrap();

                            let bounded_timecode = if timecode < 0.0 {
                                0.0
                            } else if timecode > video_meta.duration {
                                video_meta.duration
                            } else {
                                timecode
                            };

                            let video_with_computed_poster = compute_poster(&context, bounded_timecode, video_with_metadata);

                            if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_computed_poster) {
                                context.processing_errors.lock().unwrap().push(err);
                            }
                        } else {
                            let message = format!("There was no source video to compute a poster from. ({})", video_with_metadata.site_path.normalized());
                            context.processing_errors.lock().unwrap().push(message);
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                }

                context.complete_job(&job.id);
            }
        });
    }
}

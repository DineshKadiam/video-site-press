use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{
    Context,
    FileMeta,
    ImageProcessor,
    JobKind,
    Playlist,
    PosterAsset,
    PosterAssets,
    PosterAssetsCached,
    PosterFile,
    ResizeMode,
    Video,
    VideoMetaCached
};
use crate::transcode;

fn compute_banner_assets(context: &Context, playlist: Arc<Playlist>) -> Result<(), String> {
    if let Some(banner) = &playlist.banner {
        let source_path = context.site_dir.join(&playlist.path).join(&banner.file_name);

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
                banner_assets.write_cache_manifest(context, &banner.file_meta, &banner.file_name, &playlist.path);

                // Add/update poster assets in in-memory cache
                // TODO: This looks vaguely in need of encapsulation
                if let Ok(mut cache) = context.cache.lock() {
                    let banner_path = format!("{}/{}", playlist.path, banner.file_name);

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

                let mutation = |playlist: &mut Playlist| {
                    // Note that we expect the banner to be Some(), virtually always.
                    // If it's missing that is an edge case with (likely) user interaction.
                    if let Some(banner) = &mut playlist.banner {
                        banner.assets = Some(banner_assets);
                    }
                };

                if let Err(()) = context.update_playlist(mutation, &playlist.path) {
                    return Err("Banner assets could not be assigned in the site tree as the playlist has meanwhile changed path.".to_string());
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

        if let Ok(video_meta) = transcode::ffprobe(&context.site_dir.join(&video.path).join(&version.file_name)) {
            video_meta.write_cache_manifest(context, &version.file_meta, &version.file_name, &video.path);

            // Add/update video meta in in-memory cache
            // TODO: This looks vaguely in need of encapsulation
            if let Ok(mut cache) = context.cache.lock() {
                let version_path = format!("{}/{}", &video.path, version.file_name);

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
            video_mut.versions = versions_mut;
        };

        if let Ok(video_updated) = context.update_video(mutation, &video.path) {
            // TODO: Reactivate
            // context.create_version_jobs(&video);
            return video_updated;
        }
    }

    video
}

fn compute_poster(context: &Context, seek_offset: f32, video: Arc<Video>) -> Arc<Video> {
    if let Ok(file_name) = transcode::compute_poster(context, seek_offset, &video) {
        let file_path = context.site_dir.join(&video.path).join(&file_name);
        let file_meta = FileMeta::new(&file_path);
        let poster_file = PosterFile::new(None, None, file_meta, file_name);

        let mutation = |video_mut: &mut Video| {
            video_mut.poster = Some(poster_file);
        };

        if let Ok(video_updated) = context.update_video(mutation, &video.path) {
            video_updated.write_manifest(context);
            return video_updated;
        }
    }

    video
}

fn compute_poster_asset(context: &Context, source_image: &ImageProcessor, resize_mode: ResizeMode) -> Result<PosterAsset, String> {
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
        let source_path = context.site_dir.join(&video.path).join(&poster.file_name);

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

                poster_assets.write_cache_manifest(context, &poster.file_meta, &poster.file_name, &video.path);

                // Add/update poster assets in in-memory cache
                // TODO: This looks vaguely in need of encapsulation
                if let Ok(mut cache) = context.cache.lock() {
                    let poster_path = format!("{}/{}", video.path, poster.file_name);

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
                    }
                };

                if let Err(()) = context.update_video(mutation, &video.path) {
                    return Err("Poster assets could not be assigned in the site tree as the video has meanwhile changed path.".to_string());
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
                    JobKind::PlaylistFastTasks(playlist) => {
                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &playlist.path);
                        let process_id = context.processing.lock().unwrap().push(&playlist.path, label);

                        if let Some(banner) = &playlist.banner {
                            if banner.assets.is_none() {
                                if let Err(err) = compute_banner_assets(&context, playlist) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoFastTasks { poster_aspect, video } => {
                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &video.path);
                        let process_id = context.processing.lock().unwrap().push(&video.path, label);

                        let video_with_metadata = compute_metadata(&context, video);

                        if let Some(poster) = &video_with_metadata.poster {
                            if poster.assets.is_none() {
                                if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_metadata) {
                                    context.processing_errors.lock().unwrap().push(err);
                                }
                            }
                        } else {
                            let video_with_poster = compute_poster(&context, 0.5, video_with_metadata);

                            if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_poster) {
                                context.processing_errors.lock().unwrap().push(err);
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoFormat { format, video } => {
                        let format_label = format.label(); // Implement Display so we can just drop it right into interpolation?
                        let label = format!("[Worker {worker_number}] Computing version {format_label} for {}", video.path);
                        let process_id = context.processing.lock().unwrap().push(&video.path, label);

                        match transcode::encode(&context, format, process_id, &video) {
                            Ok(version) => {
                                let mutation = |video_mut: &mut Video| {
                                    video_mut.versions.push(version);
                                };

                                match context.update_video(mutation, &video.path) {
                                    Ok(video_updated) => {
                                        let site = context.get_site();
                                        let job_kind = JobKind::VideoFastTasks {
                                            poster_aspect: site.poster_aspect.numeric(),
                                            video: Arc::clone(&video_updated)
                                        };
                                        context.push_job_queue(job_kind);
                                    }
                                    Err(()) => context.processing_errors.lock().unwrap().push("Video could not be updated with newly computed version.".to_string())
                                }
                            }
                            Err(()) => {
                                context.processing_errors.lock().unwrap().push("Encoding a new version failed for an unknown reason.".to_string())
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::VideoRandomPoster { poster_aspect, video } => {
                        let label = format!("[Worker {worker_number}] Computing random poster for {}", &video.path);
                        let process_id = context.processing.lock().unwrap().push(&video.path, label);

                        let video_with_metadata = compute_metadata(&context, video);

                        let seek_offset = rand::random::<f32>(); // Produces a value between 0.0-0.1
                        let video_with_random_poster = compute_poster(&context, seek_offset, video_with_metadata);

                        if let Err(err) = compute_poster_assets(&context, poster_aspect, video_with_random_poster) {
                            context.processing_errors.lock().unwrap().push(err);
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                }

                context.complete_job(&job.id);
            }
        });
    }
}
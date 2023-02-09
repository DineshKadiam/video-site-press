use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{
    Context,
    FileMeta,
    ImageProcessor,
    JobKind,
    PosterAsset,
    PosterAssets,
    PosterFile,
    ResizeMode,
    Video,
    VideoMetaCached
};
use crate::transcode;

fn compute_poster(context: &Context, video: &Video) {
    if let Ok(file_name) = transcode::compute_poster(&context, video) {
        let mutation = |video: &mut Video| {
            let file_path = context.site_dir.join(&video.path).join(&file_name);
            let file_meta = FileMeta::new(&file_path);

            let poster_file = PosterFile::new(None, file_meta, file_name);

            video.poster = Some(poster_file)
        };

        if let Ok(video) = context.update_video(mutation, &video.path) {
            video.persist_in_cache(&context.cache_dir);
            video.persist_in_site(&context);
        }
    } 
}

fn compute_poster_asset(context: &Context, source_image: &ImageProcessor, width: u32) -> PosterAsset {
    let (file_name, dimensions) = source_image.resize(
        context,
        ResizeMode::CoverRectangle { aspect: 1.7777777, max_width: width }
    );

    let metadata = fs::metadata(context.cache_dir.join(&file_name)).unwrap();

    PosterAsset {
        file_name,
        filesize_bytes: metadata.len(),
        height: dimensions.1,
        width: dimensions.0
    }
}

fn compute_poster_assets(context: &Context, video: &Video) {
    if let Some(poster) = &video.poster {
        let source_path = context.site_dir.join(&video.path).join(&poster.file_name);

        let source_processor = ImageProcessor::open(&source_path);

        let min_overshoot = 1.1;

        let max_160 = compute_poster_asset(context, &source_processor, 160);

        let max_320 = if source_processor.source_width as f32 > 160.0 * min_overshoot {
            Some(compute_poster_asset(context, &source_processor, 320))
        } else {
            None
        };

        let max_480 = if source_processor.source_width as f32 > 320.0 * min_overshoot {
            Some(compute_poster_asset(context, &source_processor, 480))
        } else {
            None
        };

        let max_800 = if source_processor.source_width as f32 > 480.0 * min_overshoot {
            Some(compute_poster_asset(context, &source_processor, 800))
        } else {
            None
        };

        let max_1280 = if source_processor.source_width as f32 > 800.0 * min_overshoot {
            Some(compute_poster_asset(context, &source_processor, 1280))
        } else {
            None
        };

        let poster_assets = PosterAssets {
            max_160,
            max_320,
            max_480,
            max_800,
            max_1280
        };

        let mutation = |video: &mut Video| {
            // Note that we expect the poster to be Some(), virtually always.
            // If it's missing that is an edge case with (likely) user interaction.
            if let Some(poster) = &mut video.poster {
                poster.assets = Some(poster_assets);
            }
        };

        if let Ok(video) = context.update_video(mutation, &video.path) {
            video.persist_in_cache(&context.cache_dir);
            video.persist_in_site(&context);
        }
    } 
}

pub fn start_threads(count: usize, context_ref: &Arc<Context>) {
	for index in 0..count {
        let context = Arc::clone(context_ref);

        thread::spawn(move || {
            let worker_number = index + 1;
            let high_priority_only = worker_number == 1;

            println!("[Worker {worker_number}] Started");
            loop {
                let job = context.take_processing_job(high_priority_only);

                if job.is_none() {
                    thread::sleep(Duration::from_millis(1000));
                    continue
                }

                let job = job.unwrap();

                match job.kind {
                    JobKind::Metadata => {
                        let label = format!("[Worker {worker_number}] Probing metadata for versions of {}", job.video.path);

                        let process_id = context.processing.lock().unwrap().push(&job.video.path, label);

                        let mut versions_mut = job.video.versions.clone();

                        for version in versions_mut.iter_mut() {
                            if version.video_meta.is_some() { continue }

                            if let Ok(video_meta) = transcode::ffprobe(&context.site_dir.join(&job.video.path).join(&version.file_name)) {
                                let version_path = format!("{}/{}", &job.video.path, version.file_name);
                                let video_meta_cached = VideoMetaCached {
                                    file_meta: version.file_meta.clone(),
                                    path: version_path,
                                    video_meta: video_meta.clone()
                                };

                                context.cache.write().unwrap().video_meta.push(video_meta_cached);

                                version.video_meta = Some(video_meta);
                            }
                        }

                        let mutation = |video: &mut Video| {
                            video.versions = versions_mut;
                        };

                        if let Ok(video) = context.update_video(mutation, &job.video.path) {
                            video.persist_in_cache(&context.cache_dir);

                            // TODO: Reactivate
                            // context.create_version_jobs(&video);
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::Poster => {
                        let label = format!("[Worker {worker_number}] Computing poster for {}", job.video.path);
                        let process_id = context.processing.lock().unwrap().push(&job.video.path, label);

                        if let Some(poster) = &job.video.poster {
                            if poster.assets.is_none() {
                                compute_poster_assets(&context, &job.video);
                            }
                        } else {
                            compute_poster(&context, &job.video);
                            compute_poster_assets(&context, &job.video);
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::Format(format) => {
                        let format_label = format.label(); // Implement Display so we can just drop it right into interpolation?
                        let label = format!("[Worker {worker_number}] Computing version {format_label} for {}", job.video.path);
                        let process_id = context.processing.lock().unwrap().push(&job.video.path, label);

                        if let Ok(version) = transcode::encode(&context, format, &job.video) {
                            let mutation = |video: &mut Video| {
                                // TODO: Reactivate
                                // video.assets.versions.push(version);
                            };

                            if let Ok(video) = context.update_video(mutation, &job.video.path) {
                                video.persist_in_cache(&context.cache_dir);
                            }
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                }

                context.complete_job(&job.id);
            }
        });
    }
}
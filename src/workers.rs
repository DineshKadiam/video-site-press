use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{
    Context,
    FileMeta,
    ImageProcessor,
    Job,
    JobKind,
    PosterAsset,
    PosterAssets,
    PosterFile,
    ResizeMode,
    Video,
    VideoMetaCached
};
use crate::transcode;

fn compute_metadata(context: &Context, video: Arc<Video>) -> Arc<Video> {
    let mut versions_mut = video.versions.clone();

    let mut made_changes = false;

    for version in versions_mut.iter_mut() {
        if version.video_meta.is_some() { continue }

        if let Ok(video_meta) = transcode::ffprobe(&context.site_dir.join(&video.path).join(&version.file_name)) {
            let version_path = format!("{}/{}", &video.path, version.file_name);
            let video_meta_cached = VideoMetaCached {
                file_meta: version.file_meta.clone(),
                path: version_path,
                video_meta: video_meta.clone()
            };

            context.cache.lock().unwrap().video_meta.push(video_meta_cached);

            version.video_meta = Some(video_meta);

            made_changes = true;
        }
    }

    if made_changes {
        let mutation = |video_mut: &mut Video| {
            video_mut.versions = versions_mut;
        };

        if let Ok(video_updated) = context.update_video(mutation, &video.path) {
            video_updated.persist_in_cache(&context.cache_dir);

            // TODO: Reactivate
            // context.create_version_jobs(&video);

            return video_updated;
        }
    }

    video
}

fn compute_poster(context: &Context, video: Arc<Video>) -> Arc<Video> {
    if let Ok(file_name) = transcode::compute_poster(&context, &video) {
        let file_path = context.site_dir.join(&video.path).join(&file_name);
        let file_meta = FileMeta::new(&file_path);
        let poster_file = PosterFile::new(None, file_meta, file_name);

        let mutation = |video_mut: &mut Video| {
            video_mut.poster = Some(poster_file);
        };

        if let Ok(video_updated) = context.update_video(mutation, &video.path) {
            video_updated.write_manifest(&context);
            return video_updated;
        }
    }

    video
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

fn compute_poster_assets(context: &Context, video: Arc<Video>) {
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

        poster_assets.write_manifest(context, &video.path, poster);

        let mutation = |video: &mut Video| {
            // Note that we expect the poster to be Some(), virtually always.
            // If it's missing that is an edge case with (likely) user interaction.
            if let Some(poster) = &mut video.poster {
                poster.assets = Some(poster_assets);
            }
        };

        if let Ok(video) = context.update_video(mutation, &video.path) {
            video.write_manifest(&context);
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
                    JobKind::FastTasks => {
                        let video: Arc<Video> = job.video;

                        let label = format!("[Worker {worker_number}] Doing fast track processing for {}", &video.path);
                        let process_id = context.processing.lock().unwrap().push(&video.path, label);

                        let video_with_metadata = compute_metadata(&context, video);

                        if let Some(poster) = &video_with_metadata.poster {
                            if poster.assets.is_none() {
                                compute_poster_assets(&context, video_with_metadata);
                            }
                        } else {
                            let video_with_poster = compute_poster(&context, video_with_metadata);
                            compute_poster_assets(&context, video_with_poster);
                        }

                        context.processing.lock().unwrap().remove(process_id);
                    }
                    JobKind::Format(format) => {
                        let format_label = format.label(); // Implement Display so we can just drop it right into interpolation?
                        let label = format!("[Worker {worker_number}] Computing version {format_label} for {}", job.video.path);
                        let process_id = context.processing.lock().unwrap().push(&job.video.path, label);

                        if let Ok(version) = transcode::encode(&context, format, &job.video) {
                            let mutation = |video: &mut Video| {
                                video.versions.push(version);
                            };

                            if let Ok(video) = context.update_video(mutation, &job.video.path) {
                                video.persist_in_cache(&context.cache_dir);
                            }
                        }
                        // TODO: Temporary hack - for now we don't set video_metadata in transcode::encode (although we totally could, see TODO in there)
                        //       Instead of appending a FastTasks job, fill video metadata in transcode::encode and remove this job creation here.
                        let follow_up_job = Job::new(JobKind::FastTasks, Arc::clone(&job.video));
                        context.job_queue.lock().unwrap().push(follow_up_job);

                        context.processing.lock().unwrap().remove(process_id);
                    }
                }

                context.complete_job(&job.id);
            }
        });
    }
}
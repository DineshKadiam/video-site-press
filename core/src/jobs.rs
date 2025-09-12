// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::Context;

mod encode;
mod job;
mod job_desc;
mod job_error;
mod job_progress;
mod probe;
mod worker;

pub use job::{CompletedJob, QueuedJob};
pub use job_desc::JobDesc;
pub use job_error::JobError;
pub use job_progress::JobProgress;
pub use worker::Worker;

use encode::FFmpegEncodingStats;

pub const MIN_WORKERS: usize = 2;

pub fn start_workers(
    context_ref: &Arc<Context>,
    custom_count: Option<usize>
) {
    let count = custom_count
        .map(|count| count.max(MIN_WORKERS))
        .unwrap_or(MIN_WORKERS);

    for worker_number in 1..=count {
        let context = Arc::clone(context_ref);

        thread::spawn(move || {
            let high_priority_only = worker_number == 1;

            let worker = Worker::new(high_priority_only, worker_number);

            loop {
                let job = match context.take_next_job(&worker) {
                    Some(job) => job,
                    None => {
                        thread::sleep(Duration::from_millis(1000));
                        continue
                    }
                };

                let job_error = match job.desc {
                    JobDesc::AudioFormat { format, video } => {
                        Worker::render_audio_format(
                            format,
                            &context,
                            job.id.clone(),
                            video
                        ).err()
                    }
                    JobDesc::AudioMeta { video } => {
                        Worker::compute_audio_metadata(
                            &context,
                            video
                        ).err()
                    }
                    JobDesc::CollectionFastTasks { collection } => {
                        Worker::compute_collection_fast_tasks(
                            collection,
                            &context
                        ).err()
                    }
                    JobDesc::PlaylistFastTasks { playlist } => {
                        Worker::compute_playlist_fast_tasks(
                            &context,
                            playlist
                        ).err()
                    }
                    JobDesc::VideoFastTasks { poster_aspect, video } => {
                        Worker::compute_video_fast_tasks(
                            &context,
                            poster_aspect,
                            video
                        ).err()
                    }
                    JobDesc::VideoFormat { format, video } => {
                        Worker::render_video_format(
                            &context,
                            job.id.clone(),
                            video,
                            format
                        ).err()
                    }
                    JobDesc::VideoPickPoster { poster_aspect, timecode, video } => {
                        Worker::pick_video_poster(
                            &context,
                            poster_aspect,
                            timecode,
                            video
                        ).err()
                    }
                };

                if let Some(error) = job_error {
                    context.fail_job(error, &job.id);
                } else {
                    context.complete_job(&job.id);
                }

                // TODO: Maybe we want to introduce retries (a defined number
                // of retries, e.g. 2 or 3) here (but we should consider for
                // which cases this would makes sense, no implementation for
                // this without some reasonable justification)
            }
        });
    }
}

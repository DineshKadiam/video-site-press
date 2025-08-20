// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::Context;

mod active_jobs;
mod encode;
mod job;
mod job_kind;
mod job_progress;
mod probe;
mod worker;

pub use active_jobs::ActiveJobs;
pub use job::Job;
pub use job_kind::JobKind;
pub use job_progress::JobProgress;

use encode::FFmpegEncodingStats;

pub fn start_workers(count: usize, context_ref: &Arc<Context>) {
    for index in 0..count {
        let context = Arc::clone(context_ref);

        thread::spawn(move || {
            let worker_number = index + 1;
            let high_priority_only = worker_number == 1;

            let language = context.get_editor_language();
            let translations = &language.translations;

            loop {
                let job = match context.take_next_job(high_priority_only) {
                    Some(job) => job,
                    None => {
                        thread::sleep(Duration::from_millis(1000));
                        continue
                    }
                };

                let job_kind_label = job.kind.label(translations);
                let worker_job_label = format!("[Worker {worker_number}] {job_kind_label}");
                let site_path = job.kind.site_path();
                let job_id = context.add_active_job(worker_job_label, site_path.clone());

                let job_error = match job.kind {
                    JobKind::AudioFormat { format, video } => {
                        worker::render_audio_format(
                            format,
                            &context,
                            job_id,
                            translations,
                            video
                        ).err()
                    }
                    JobKind::AudioMeta { video } => {
                        worker::compute_audio_metadata(
                            &context,
                            translations,
                            video
                        ).err()
                    }
                    JobKind::CollectionFastTasks { collection } => {
                        worker::compute_collection_fast_tasks(
                            collection,
                            &context,
                            translations
                        ).err()
                    }
                    JobKind::PlaylistFastTasks { playlist } => {
                        worker::compute_playlist_fast_tasks(
                            &context,
                            playlist,
                            translations
                        ).err()
                    }
                    JobKind::VideoFastTasks { poster_aspect, video } => {
                        worker::compute_video_fast_tasks(
                            &context,
                            poster_aspect,
                            translations,
                            video
                        ).err()
                    }
                    JobKind::VideoFormat { format, video } => {
                        worker::render_video_format(
                            &context,
                            job_id,
                            translations,
                            video,
                            format
                        ).err()
                    }
                    JobKind::VideoPickPoster { poster_aspect, timecode, video } => {
                        worker::pick_video_poster(
                            &context,
                            poster_aspect,
                            timecode,
                            translations,
                            video
                        ).err()
                    }
                };

                // TODO: Maybe we want to introduce retries (a defined number
                // of retries, e.g. 2 or 3) here (but we should consider for
                // which cases this would makes sense, no implementation for
                // this without some reasonable justification)
                if let Some(error) = job_error {
                    context.job_error(error);
                }

                context.remove_active_job(job_id);
                context.complete_job(&job.id);
            }
        });
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use actix_session::Session;
use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;

use hyper8_core::{
    FileMeta,
    InfallibleResult,
    JobDesc,
    PosterFile,
    SitePath,
    Video,
    VideoUpdate
};
use hyper8_core::infallible_mutation;
use hyper8_core::util::{nanoid, supported_image_extension};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::edit_poster::edit_poster_page;
use super::edit_poster::UploadPosterFormFeedback;

use super::pending_manifest_errors;

pub async fn upload_poster(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let session_state = SessionState::all(&context.editor_state, &session);

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                if !supported_image_extension(filename) {
                    match context.get_video(&site_path) {
                        Some(video) => {
                            let error = format!("Unsupported file extension in {filename}");
                            let form_feedback = UploadPosterFormFeedback { error };
                            let http_response = edit_poster_page(&context, Some(form_feedback), &session_state, &video);
                            return Ok(Either::Right(http_response))
                        }
                        None => {
                            let http_response = not_found(&context, &session_state, &site_path);
                            return Ok(Either::Right(http_response));
                        }
                    }
                }

                let original_filename = filename.to_string();

                let storage_filename = if !context.site_dir
                    .join(site_path.filesystem_relative())
                    .join(&original_filename)
                    .exists() {
                    original_filename
                } else {
                    format!("{}_{original_filename}", nanoid())
                };

                let storage_path = context.site_dir
                    .join(site_path.filesystem_relative())
                    .join(&storage_filename);

                let storage_path_moving = storage_path.clone();

                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| File::create(storage_path_moving)).await??;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                let file_meta = FileMeta::new(&storage_path);
                let poster_file = PosterFile::new(
                    None,
                    None,
                    file_meta,
                    storage_filename,
                    None
                );

                let mutation = |video: &mut Video| -> InfallibleResult {
                    video.poster = Some(poster_file);
                    video.write_manifest(&context);

                    Ok(())
                };

                match context.update_video(mutation, &site_path) {
                    VideoUpdate::Error(_) => infallible_mutation!(),
                    VideoUpdate::InvalidManifest { video } => {
                        return Ok(Either::Right(pending_manifest_errors(&context, &session_state, &video)))
                    }
                    VideoUpdate::NotFound => {
                        let _ = fs::remove_file(&storage_path);
                        let http_response = not_found(&context, &session_state, &site_path);
                        return Ok(Either::Right(http_response));
                    }
                    VideoUpdate::Success { video_updated, .. } => {
                        let site = context.get_site();
                        let video_fast_tasks_job_desc = JobDesc::VideoFastTasks {
                            poster_aspect: site.poster_aspect.numeric(),
                            video: Arc::clone(&video_updated)
                        };
                        context.queue_job(video_fast_tasks_job_desc);

                        let redirect_url = routes::video_poster(&video_updated);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()));
                    }
                }
            }
        }
    }

    unreachable!()
}

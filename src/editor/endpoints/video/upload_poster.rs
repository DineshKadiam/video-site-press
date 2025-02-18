// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;

use crate::{
    Context,
    ContextPath,
    FileMeta,
    JobKind,
    PosterFile,
    Video
};
use crate::editor::widgets::not_found;
use crate::util::{nanoid, supported_image_extension};

use super::edit_page;
use super::VideoFormFeedback;

pub async fn upload_poster(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                if !supported_image_extension(filename) {
                    match context.get_video(&path.path) {
                        Some(video) => {
                            let error = format!("Unsupported file extension in {filename}");
                            let form_feedback = VideoFormFeedback::UploadPoster { error };
                            let http_response = edit_page(&context, Some(form_feedback), &language, &video);
                            return Ok(Either::Right(http_response))
                        }
                        None => {
                            let http_response = not_found(&context, &language, &path.path);
                            return Ok(Either::Right(http_response));
                        }
                    }
                }

                let original_filename = filename.to_string();

                let storage_filename = if !context.site_dir.join(&path.path).join(&original_filename).exists() {
                    original_filename
                } else {
                    format!("{}_{original_filename}", nanoid())
                };

                let storage_path = context.site_dir.join(&path.path).join(&storage_filename);
                let storage_path_moving = storage_path.clone();

                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| File::create(storage_path_moving)).await??;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                let file_meta = FileMeta::new(&storage_path);
                let poster_file = PosterFile::new(None, None, file_meta, storage_filename);

                let mutation = |video: &mut Video| {
                    video.poster = Some(poster_file);
                    video.write_manifest(&context);
                };

                let video = match context.update_video(mutation, &path.path) {
                    Ok(video) => video,
                    Err(()) => {
                        let _ = fs::remove_file(&storage_path);
                        let http_response = not_found(&context, &language, &path.path);
                        return Ok(Either::Right(http_response));
                    }
                };

                let site = context.get_site();
                let job_kind = JobKind::VideoFastTasks {
                    poster_aspect: site.poster_aspect.numeric(),
                    video: Arc::clone(&video)
                };
                context.push_job_queue(job_kind);

                let redirect_url = format!("/video/{}", path.path);
                return Ok(Either::Left(Redirect::to(redirect_url).see_other()));
            }
        }
    }

    unreachable!()
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    SitePath,
    SubtitleFile,
    Video
};
use crate::editor::routes;
use crate::editor::widgets::{error_page, not_found};
use crate::util::{deduplicate_filename, supported_subtitles_extension};

pub async fn upload_subtitles(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    site_path: ActixPath<SitePath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    let video = match context.get_video(&site_path) {
        Some(video) => video,
        None => {
            let http_response = not_found(&context, &language, &site_path);
            return Ok(Either::Right(http_response));
        }
    };

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                if supported_subtitles_extension(filename) {
                    let mut candidate_filename = filename.to_string();
                    let mut storage_path = context.site_dir
                        .join(video.site_path.filesystem_relative())
                        .join(&candidate_filename);

                    while storage_path.exists() {
                        candidate_filename = deduplicate_filename(&candidate_filename);
                        storage_path = context.site_dir
                            .join(video.site_path.filesystem_relative())
                            .join(&candidate_filename);
                    }

                    let storage_path_moving = storage_path.clone();
                    // File::create is blocking operation, use threadpool
                    let mut f = web::block(|| File::create(storage_path_moving)).await??;

                    // Field in turn is stream of *Bytes* object
                    while let Some(chunk) = field.try_next().await? {
                        // filesystem operations are blocking, we have to use threadpool
                        f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                    }

                    let mutation = |video: &mut Video| {
                        video.subtitles.push(SubtitleFile::from_path(&storage_path));
                    };

                    match context.update_video(mutation, &site_path) {
                        Ok(video) => {
                            let redirect_url = routes::video_subtitles(&video);
                            return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                        }
                        // TODO: Can only happen in weird time of check vs. time of use edge cases
                        Err(()) => return Ok(Either::Right(not_found(&context, &language, &site_path)))
                    }
                } else {
                    // TODO: Translate
                    let message = format!("The subtitle extension of '{filename}' is not supported, please upload only .srt or .vtt files.");
                    return Ok(Either::Right(error_page(&context, &language, &message)))
                }
            }
        }
    }

    unimplemented!()
}

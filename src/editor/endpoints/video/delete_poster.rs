// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, ContextPath, Video};
use crate::editor::widgets::not_found;

pub async fn delete_poster(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(poster) = &video_mut.poster {
            let poster_path = context.site_dir.join(&video_mut.path).join(&poster.file_name);
            video_mut.poster = None;
            video_mut.write_manifest(&context);
            let _ = fs::remove_file(poster_path);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

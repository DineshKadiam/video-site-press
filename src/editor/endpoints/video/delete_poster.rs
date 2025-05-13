// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath, Video, VideoUpdate};
use crate::editor::routes;
use crate::editor::widgets::not_found;

use super::pending_manifest_errors;

pub async fn delete_poster(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(poster) = &video_mut.poster {
            let poster_path = context.site_dir
                .join(video_mut.site_path.filesystem_relative())
                .join(&poster.file_name);

            video_mut.poster = None;
            video_mut.write_manifest(&context);
            let _ = fs::remove_file(poster_path);
        }

        Ok(())
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(_) => unreachable!("The mutation is infallible"),
        VideoUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
        VideoUpdate::PendingErrors { video } => {
            let language = context.get_editor_language();
            Either::Right(pending_manifest_errors(&context, &language, &video))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_poster(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

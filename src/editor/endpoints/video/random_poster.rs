// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, ContextPath, JobKind};
use crate::editor::widgets::not_found;

pub async fn random_poster(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&path.path) {
        Some(video) => {
            let site = context.get_site();
            let job_kind = JobKind::VideoRandomPoster {
                poster_aspect: site.poster_aspect.numeric(),
                video: Arc::clone(&video)
            };
            context.push_job_queue(job_kind);

            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

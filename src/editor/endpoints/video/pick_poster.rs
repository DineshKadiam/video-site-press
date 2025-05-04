// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Context,
    JobKind,
    SitePath
};
use crate::editor::routes;
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct PickPosterForm {
    timecode: String
}

pub async fn pick_poster(
    context: Data<Arc<Context>>,
    form: Form<PickPosterForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&site_path) {
        Some(video) => {
            let site = context.get_site();

            let poster_aspect = site.poster_aspect.numeric();
            let timecode = form.timecode.parse::<f32>().unwrap();

            let job_kind = JobKind::VideoPickPoster {
                poster_aspect,
                timecode,
                video: Arc::clone(&video)
            };
            context.push_job_queue(job_kind);

            let redirect_url = routes::video_poster(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
    }
}

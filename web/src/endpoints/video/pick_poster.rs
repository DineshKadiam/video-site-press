// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    JobDesc,
    SitePath
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

#[derive(Deserialize)]
pub struct PickPosterForm {
    timecode: String
}

pub async fn pick_poster(
    context: Data<Arc<Context>>,
    form: Form<PickPosterForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&site_path) {
        Some(video) => {
            let site = context.get_site();

            let poster_aspect = site.poster_aspect.numeric();
            let timecode = form.timecode.parse::<f32>().unwrap();

            let pick_poster_job_desc = JobDesc::VideoPickPoster {
                poster_aspect,
                timecode,
                video: Arc::clone(&video)
            };

            context.queue_job(pick_poster_job_desc);

            let redirect_url = routes::video_poster(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
    }
}

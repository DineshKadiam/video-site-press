// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    JobDesc,
    SitePath,
    VideoFormat
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

#[derive(Deserialize)]
pub struct EncodeVideoForm {
    approach: String,
    codec: String,
    frame_rate: String,
    height: String,
    quality: String
}

/// Request a new (resized/transcoded) video file for this video to be encoded
pub async fn encode_video(
    context: Data<Arc<Context>>,
    form: Form<EncodeVideoForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&site_path) {
        Some(video) => {
            let frame_rate = if form.frame_rate.is_empty() {
                None
            } else {
                Some(form.frame_rate.as_str())
            };

            let video_format = VideoFormat::new(
                &form.codec,
                "mp4",
                &form.approach,
                frame_rate,
                &form.height,
                &form.quality,
            ).unwrap();

            let video_format_job_desc = JobDesc::VideoFormat {
                format: video_format,
                video: Arc::clone(&video)
            };

            context.queue_job(video_format_job_desc);

            let redirect_url = routes::video_media(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Context,
    JobKind,
    SitePath,
    VideoFormat
};
use crate::editor::routes;
use crate::editor::widgets::not_found;

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

            let job_kind = JobKind::VideoFormat { format: video_format, video: Arc::clone(&video) };
            context.queue_job(job_kind);

            let redirect_url = routes::video_media(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
    }
}

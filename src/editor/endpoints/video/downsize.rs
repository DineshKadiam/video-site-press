// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, ContextPath, JobKind, VideoFormat};
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct DownsizeForm {
    approach: String,
    codec: String,
    frame_rate: String,
    height: String,
    quality: String
}

/// Request a new version the video to be encoded
pub async fn downsize(
    context: Data<Arc<Context>>,
    form: Form<DownsizeForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
    };

    let frame_rate = if form.frame_rate.is_empty() {
        None
    } else {
        Some(form.frame_rate.as_str())
    };

    let video_format = VideoFormat::new_video(
        &form.codec,
        "mp4",
        &form.approach,
        frame_rate,
        &form.height,
        &form.quality,
    ).unwrap();

    let job_kind = JobKind::VideoFormat { format: video_format, video: Arc::clone(&video) }; 
    context.push_job_queue(job_kind);

    let redirect_url = format!("/video/{}", &path.path);
    Either::Left(Redirect::to(redirect_url).see_other())
}

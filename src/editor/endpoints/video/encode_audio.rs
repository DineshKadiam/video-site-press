// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    AudioFormat,
    Context,
    JobKind,
    SitePath
};
use crate::editor::routes;
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct EncodeAudioForm {
    format: String
}

/// Request the video to be encoded as an audio-only file
pub async fn encode_audio(
    context: Data<Arc<Context>>,
    form: Form<EncodeAudioForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&site_path) {
        Some(video) => {
            let format = AudioFormat::from_key(&form.format).unwrap();

            let job_kind = JobKind::AudioFormat {
                format,
                video: Arc::clone(&video)
            };
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

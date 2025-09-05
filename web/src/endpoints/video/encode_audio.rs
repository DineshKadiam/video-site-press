// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    AudioFormat,
    JobDesc,
    SitePath
};

use crate::{Context, Session};
use crate::routes;
use crate::widgets::not_found;

#[derive(Deserialize)]
pub struct EncodeAudioForm {
    format: String
}

/// Request the video to be encoded as an audio-only file
pub async fn encode_audio(
    context: Data<Arc<Context>>,
    form: Form<EncodeAudioForm>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&site_path) {
        Some(video) => {
            let format = AudioFormat::from_key(&form.format).unwrap();

            let audio_format_job_desc = JobDesc::AudioFormat {
                format,
                video: Arc::clone(&video)
            };
            context.queue_job(audio_format_job_desc);

            let redirect_url = routes::video_media(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
    }
}

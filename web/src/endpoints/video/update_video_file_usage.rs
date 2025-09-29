// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    MediaUsage,
    SitePath,
    Video,
    VideoUpdate
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct UpdateVideoFileUsageForm {
    file_name: String,
    usage: String
}

pub async fn update_video_file_usage(
    context: Data<Arc<Context>>,
    form: Form<UpdateVideoFileUsageForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        let usage = MediaUsage::from_key_unchecked(&form.usage);

        if let Some(video_file) = video_mut.video_files
            .iter_mut()
            .find(|video_file| video_file.file_name == form.file_name) {
            video_file.usage = usage;
            video_mut.write_manifest(&context);

            Ok(())
        } else {
            Err(())
        }
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(()) => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        VideoUpdate::InvalidManifest { video } => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(pending_manifest_errors(&context, &session_state, &video))
        }
        VideoUpdate::NotFound => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_media(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

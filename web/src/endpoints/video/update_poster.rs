// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    SitePath,
    Video,
    VideoUpdate
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::{error_page, not_found};

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct PosterUpdateForm {
    description: String
}

pub async fn update_poster(
    context: Data<Arc<Context>>,
    form: Form<PosterUpdateForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video: &mut Video| {
        if let Some(poster) = &mut video.poster {
            poster.description = Some(form.description.clone());
            video.write_manifest(&context);

            Ok(())
        } else {
            let session_language = SessionState::language(&context.server_mode, &session);
            Err(session_language.translations.the_poster_to_be_updated_was_not_found.to_string())
        }
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(err) => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(error_page(&context, &err, &session_state))
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
            let redirect_url = routes::video_poster(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

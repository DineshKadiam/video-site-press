// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{SitePath, Video, VideoUpdate};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::{error_page, not_found};

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct DeleteSubtitlesForm {
    file_name: String
}

pub async fn delete_subtitles(
    context: Data<Arc<Context>>,
    form: Form<DeleteSubtitlesForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(subtitles) = video_mut.subtitles
            .iter()
            .position(|subtitles| subtitles.file_name == form.file_name)
            .map(|index| video_mut.subtitles.remove(index)) {
            let subtitles_path = context.site_dir
                .join(site_path.filesystem_relative())
                .join(&subtitles.file_name);

            let _ = fs::remove_file(&subtitles_path);
            video_mut.write_manifest(&context);

            Ok(())
        } else {
            let session_language = SessionState::language(&context.server_mode, &session);
            Err(session_language.translations.the_subtitle_requested_for_deletion_was_not_found.to_string())
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
            let redirect_url = routes::video_subtitles(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

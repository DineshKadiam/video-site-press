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
pub struct DeleteVideoFileForm {
    file_name: String
}

pub async fn delete_video_file(
    context: Data<Arc<Context>>,
    form: Form<DeleteVideoFileForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(video_file) = video_mut.video_files
            .iter()
            .position(|video_file| video_file.file_name == form.file_name)
            .map(|index| video_mut.video_files.remove(index)) {
            let version_path = context.site_dir
                .join(site_path.filesystem_relative())
                .join(&video_file.file_name);

            video_mut.write_manifest(&context);
            let _ = fs::remove_file(&version_path);

            Ok(())
        } else {
            let session_language = SessionState::language(&context.editor_state, &session);
            Err(session_language.translations.the_file_requested_for_deletion_was_not_found.to_string())
        }
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(err) => {
            let session_state = SessionState::all(&context.editor_state, &session);
            Either::Right(error_page(&context, &err, &session_state))
        }
        VideoUpdate::InvalidManifest { video } => {
            let session_state = SessionState::all(&context.editor_state, &session);
            Either::Right(pending_manifest_errors(&context, &session_state, &video))
        }
        VideoUpdate::NotFound => {
            let session_state = SessionState::all(&context.editor_state, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_media(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

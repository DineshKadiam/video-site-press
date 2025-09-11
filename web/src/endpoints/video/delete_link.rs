// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub struct DeleteLinkForm {
    label: String,
    url: String
}

pub async fn delete_link(
    context: Data<Arc<Context>>,
    form: Form<DeleteLinkForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if video_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url == form.url
                }
            )
            .map(|index| video_mut.links.remove(index))
            .is_some() {
            video_mut.write_manifest(&context);

            Ok(())
        } else {
            let session_language = SessionState::language(&context.editor_state, &session);
            Err(session_language.translations.the_link_requested_for_deletion_was_not_found.to_string())
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
            let redirect_url = routes::video_links(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

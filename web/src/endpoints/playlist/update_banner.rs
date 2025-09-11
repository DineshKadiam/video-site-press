// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    Playlist,
    PlaylistUpdate,
    SitePath
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct BannerUpdateForm {
    description: String
}

pub async fn update_banner(
    context: Data<Arc<Context>>,
    form: Form<BannerUpdateForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if let Some(banner) = &mut playlist_mut.banner {
            banner.description = Some(form.description.clone());
        }
        playlist_mut.write_manifest(&context);
    };

    match context.update_playlist(mutation, &site_path) {
        PlaylistUpdate::InvalidManifest { playlist } => {
            let session_state = SessionState::all(&context.editor_state, &session);
            Either::Right(pending_manifest_errors(&context, &playlist, &session_state))
        }
        PlaylistUpdate::NotFound => {
            let session_state = SessionState::all(&context.editor_state, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        PlaylistUpdate::Updated { playlist_updated, .. } => {
            let redirect_url = routes::playlist_banner(&playlist_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

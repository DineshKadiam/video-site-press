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
    let mutation = |playlist_mut: &mut Playlist| {
        if playlist_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url == form.url
                }
            )
            .map(|index| playlist_mut.links.remove(index))
            .is_some() {
            playlist_mut.write_manifest(&context);
        }
    };

    match context.update_playlist(mutation, &site_path) {
        PlaylistUpdate::InvalidManifest { playlist } => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(pending_manifest_errors(&context, &playlist, &session_state))
        }
        PlaylistUpdate::NotFound => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        PlaylistUpdate::Updated { playlist_updated, .. } => {
            let redirect_url = routes::playlist_links(&playlist_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

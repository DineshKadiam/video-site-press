// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use url::Url;

use hyper8_core::{
    Link,
    Playlist,
    PlaylistUpdate,
    SitePath
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::{error_page, not_found};

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct AddLinkForm {
    label: String,
    url: String,
    verification: String
}

pub async fn add_link(
    context: Data<Arc<Context>>,
    form: Form<AddLinkForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let (hidden, rel_me) = match form.verification.as_str() {
        "rel-me" => (false, true),
        "rel-me-hidden" => (true, true),
        _ => (false, false)
    };
    let label_trimmed = form.label.trim();
    let label = match label_trimmed.is_empty() {
        true => None,
        false => Some(label_trimmed.to_string())
    };
    let url = match Url::parse(&form.url) {
        Ok(_) => form.url.clone(),
        Err(err) => {
            let session_state = SessionState::all(&context.editor_state, &session);
            let t_invalid_url = session_state.language.translations.invalid_url;
            let message = format!("{t_invalid_url} ({err})");
            return Either::Right(error_page(&context, &message, &session_state))
        }
    };

    let mutation = |playlist_mut: &mut Playlist| {
        let link = Link::new(hidden, label, rel_me, url);
        playlist_mut.links.push(link);
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
            let redirect_url = routes::playlist_links(&playlist_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

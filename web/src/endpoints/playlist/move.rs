// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
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
use hyper8_core::util::validate_slug;

use crate::{Context, Field, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::dangerous_actions::dangerous_actions_page;
use super::dangerous_actions::MoveFormFeedback;
use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct PlaylistMoveForm {
    slug: String
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<PlaylistMoveForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.editor_state, &session);

    let slug_trimmed = form.slug.trim();

    let new_path = site_path
        .parent_site_path_unchecked()
        .join(slug_trimmed);

    // TODO: Here and elsewhere: If a playlist already has a permalink (say "example"),
    //       and we try to save it with exactly the same/unchanged permalink "example",
    //       we get an error that *another* playlist is already using that permalink.
    //       Need to make an exception for this in the validation check.
    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(new_path.filesystem_relative()).exists() {
        Some(String::from("This permalink is already used by another collection, playlist or video."))
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |playlist: &mut Playlist| {
            fs::rename(
                context.site_dir.join(playlist.site_path.filesystem_relative()),
                context.site_dir.join(new_path.filesystem_relative())
            ).unwrap();

            playlist.site_path = new_path;
        };

        match context.update_playlist(mutation, &site_path) {
            PlaylistUpdate::InvalidManifest { playlist } => {
                Either::Right(pending_manifest_errors(&context, &playlist, &session_state))
            }
            PlaylistUpdate::NotFound => {
                Either::Right(not_found(&context, &session_state, &site_path))
            }
            PlaylistUpdate::Updated { playlist_updated, .. } => {
                let redirect_url = routes::playlist_dangerous_actions(&playlist_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    } else {
        let playlist = match context.get_playlist(&site_path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &session_state, &site_path))
        };

        let form_feedback = MoveFormFeedback {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            dangerous_actions_page(&context, Some(form_feedback), &playlist, &session_state)
        )
    }
}

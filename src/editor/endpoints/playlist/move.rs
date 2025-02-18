// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, ContextPath, Field, Playlist};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::PlaylistFormFeedback;

#[derive(Deserialize)]
pub struct PlaylistMoveForm {
    slug: String
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<PlaylistMoveForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let new_path = Path::new(&path.path)
        .with_file_name(slug_trimmed)
        .into_os_string()
        .into_string()
        .unwrap();

    // TODO: Here and elsewhere: If a playlist already has a permalink (say "example"),
    //       and we try to save it with exactly the same/unchanged permalink "example",
    //       we get an error that *another* playlist is already using that permalink.
    //       Need to make an exception for this in the validation check.
    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&new_path).exists() {
        Some(format!("This permalink is already used by another collection, playlist or video."))
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |playlist: &mut Playlist| {
            fs::rename(
                context.site_dir.join(&playlist.path),
                context.site_dir.join(&new_path)
            ).unwrap();

            playlist.path = new_path;
        };

        let playlist = match context.update_playlist(mutation, &path.path) {
            Ok(playlist) => playlist,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = PlaylistFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &playlist)
        )
    }
}

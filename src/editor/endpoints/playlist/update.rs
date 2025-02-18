// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, ContextPath, Field, Playlist, VideoOrder};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::PlaylistFormFeedback;

#[derive(Deserialize)]
pub struct PlaylistUpdateForm {
    description: String,
    offline: Option<String>,
    order: String,
    title: String,
    unlisted: Option<String>
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<PlaylistUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let description_trimmed = form.description.trim();
    let offline = form.offline.is_some();
    let order = VideoOrder::from_key(&form.order).unwrap();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |playlist: &mut Playlist| {
            playlist.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            playlist.offline = offline;
            playlist.order = order;
            playlist.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            playlist.unlisted = unlisted;
            playlist.write_manifest(&context);
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

        let form_feedback = PlaylistFormFeedback::Update {
            description: description_trimmed,
            offline,
            order: &form.order,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &playlist)
        )
    }
}

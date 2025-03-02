// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Context,
    ContextPath,
    Field,
    OpenGraph,
    Playlist,
    PlaylistUpdate,
    VideoOrder
};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::PlaylistFormFeedback;

#[derive(Deserialize)]
pub struct PlaylistUpdateForm {
    copy_link: String,
    description: String,
    download: String,
    embedding: String,
    offline: Option<String>,
    opengraph: String,
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

    let copy_link = match form.copy_link.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

    let download = match form.download.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

    let embedding = match form.embedding.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

    let opengraph = match form.opengraph.as_str() {
        "" => None,
        other => Some(OpenGraph::from_key(other).unwrap())
    };

    let offline = form.offline.is_some();
    let order = VideoOrder::from_key(&form.order).unwrap();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |playlist_mut: &mut Playlist| {
            playlist_mut.copy_link_local_set(copy_link);
            playlist_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            playlist_mut.download_local_set(download);
            playlist_mut.embedding_local_set(embedding);
            playlist_mut.offline_local_set(offline);
            playlist_mut.opengraph_local_set(opengraph);
            playlist_mut.order = order;
            playlist_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            playlist_mut.unlisted_local_set(unlisted);
            playlist_mut.write_manifest(&context);
        };

        match context.update_playlist(mutation, &path.path) {
            PlaylistUpdate::Updated { playlist_updated, .. } => {
                let redirect_url = format!("/playlist/{}", playlist_updated.path);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
            PlaylistUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &path.path))
            }
        }
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = PlaylistFormFeedback::Update {
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            offline,
            opengraph,
            order: &form.order,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &playlist)
        )
    }
}

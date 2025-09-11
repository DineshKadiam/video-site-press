// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    PlatformIntegration,
    Playlist,
    PlaylistUpdate,
    SitePath,
    VideoOrder
};

use crate::{Context, Field, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::edit::edit_page;
use super::edit::PlaylistFormFeedback;
use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct PlaylistUpdateForm {
    copy_link: String,
    description: String,
    download: String,
    embedding: String,
    feeds: String,
    offline: Option<String>,
    order: String,
    platform_integration: String,
    sort_number: String,
    title: String,
    unlisted: Option<String>
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<PlaylistUpdateForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.editor_state, &session);

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

    let feeds = match form.feeds.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

    let offline = form.offline.is_some();
    let order = VideoOrder::from_key(&form.order).unwrap();

    let platform_integration = match form.platform_integration.as_str() {
        "" => None,
        other => Some(PlatformIntegration::from_key(other).unwrap())
    };

    let sort_number_trimmed = form.sort_number.trim();
    let sort_number = match sort_number_trimmed.is_empty() {
        true => None,
        false => Some(
            sort_number_trimmed.parse::<i32>().map_err(|err| err.to_string())
        )
    };

    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    if sort_number.as_ref().is_some_and(|sort_number| sort_number.is_err()) {
        let playlist = match context.get_playlist(&site_path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &session_state, &site_path))
        };

        let form_feedback = PlaylistFormFeedback::Update {
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            feeds,
            offline,
            order: &form.order,
            platform_integration,
            sort_number: Field::validated(sort_number_trimmed, sort_number.and_then(|sort_number| sort_number.err())),
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &playlist, &session_state)
        )
    } else {
        let mutation = |playlist_mut: &mut Playlist| {
            playlist_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            playlist_mut.order = order;
            playlist_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
            playlist_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };

            let mut must_propagate = false;

            must_propagate |= playlist_mut.copy_link_local_set(copy_link);
            must_propagate |= playlist_mut.download_local_set(download);
            must_propagate |= playlist_mut.embedding_local_set(embedding);
            must_propagate |= playlist_mut.feeds_local_set(feeds);
            must_propagate |= playlist_mut.offline_local_set(offline);
            must_propagate |= playlist_mut.platform_integration_local_set(platform_integration);
            must_propagate |= playlist_mut.unlisted_local_set(unlisted);

            if must_propagate {
                playlist_mut.propagate_options();
            }
        };

        match context.update_playlist(mutation, &site_path) {
            PlaylistUpdate::InvalidManifest { playlist } => {
                Either::Right(pending_manifest_errors(&context, &playlist, &session_state))
            }
            PlaylistUpdate::NotFound => {
                Either::Right(not_found(&context, &session_state, &site_path))
            }
            PlaylistUpdate::Updated { playlist_updated, .. } => {
                playlist_updated.write_manifest(&context);
                let redirect_url = routes::playlist_edit(&playlist_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    }
}

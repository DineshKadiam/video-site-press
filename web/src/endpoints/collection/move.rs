// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    Collection,
    CollectionUpdate,
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
pub struct MoveForm {
    slug: String
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<MoveForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.editor_state, &session);

    let slug_trimmed = form.slug.trim();

    let new_path = site_path
        .parent_site_path_unchecked()
        .join(slug_trimmed);

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(new_path.filesystem_relative()).exists() {
        Some("This permalink is already used by another collection, playlist or video.".to_string())
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |collection: &mut Collection| {
            fs::rename(
                context.site_dir.join(collection.site_path.filesystem_relative()),
                context.site_dir.join(new_path.filesystem_relative())
            ).unwrap();

            collection.site_path = new_path;
        };

        match context.update_collection(mutation, &site_path) {
            CollectionUpdate::InvalidManifest { collection } => {
                Either::Right(pending_manifest_errors(&collection, &context, &session_state))
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &session_state, &site_path))
            }
            CollectionUpdate::Updated { collection_updated, .. } => {
                let redirect_url = routes::collection_dangerous_actions(&collection_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    } else {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &session_state, &site_path))
        };

        let form_feedback = MoveFormFeedback {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            dangerous_actions_page(&collection, &context, Some(form_feedback), &session_state)
        )
    }
}

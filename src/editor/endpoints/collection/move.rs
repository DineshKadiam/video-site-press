// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    Field,
    SitePath
};
use crate::editor::routes;
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

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
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

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
                Either::Right(pending_manifest_errors(&collection, &context, &language))
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &site_path))
            }
            CollectionUpdate::Updated { collection_updated, .. } => {
                let redirect_url = routes::collection_dangerous_actions(&collection_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    } else {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = MoveFormFeedback {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            dangerous_actions_page(&collection, &context, Some(form_feedback), &language)
        )
    }
}

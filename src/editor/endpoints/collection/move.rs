// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Collection, Context, ContextPath, Field};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::CollectionFormFeedback;

#[derive(Deserialize)]
pub struct MoveForm {
    slug: String
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<MoveForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let new_path = Path::new(&path.path)
        .with_file_name(slug_trimmed)
        .into_os_string()
        .into_string()
        .unwrap();

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&new_path).exists() {
        Some("This permalink is already used by another collection, playlist or video.".to_string())
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |collection: &mut Collection| {
            fs::rename(
                context.site_dir.join(&collection.path),
                context.site_dir.join(&new_path)
            ).unwrap();

            collection.path = new_path;
        };

        let collection = match context.update_collection(mutation, &path.path) {
            Ok(collection) => collection,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &language))
    }
}

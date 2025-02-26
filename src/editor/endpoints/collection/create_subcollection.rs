// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use slug::slugify;

use crate::{Collection, Context, ContextPath, Field};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::CollectionFormFeedback;

#[derive(Deserialize)]
pub struct CreateSubcollectionForm {
    slug: String,
    title: String
}

pub async fn create_subcollection(
    context: Data<Arc<Context>>,
    form: Form<CreateSubcollectionForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();
    let title_trimmed = form.title.trim();

    let error = |slug_error, title_error| -> _ {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::CreateSubcollection {
            slug: Field::validated(slug_trimmed, slug_error),
            title: Field::validated(title_trimmed, title_error)
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &language))
    };

    if slug_trimmed.is_empty() && title_trimmed.is_empty() {
        let t_title_or_permalink_needs_to_be_provided = language.translations.title_or_permalink_needs_to_be_provided;

        return error(
            Some(t_title_or_permalink_needs_to_be_provided),
            Some(t_title_or_permalink_needs_to_be_provided)
        );
    }

    let succeed = |subcollection_path: String, title: Option<&str>| {
        let subcollection_dir = context.site_dir.join(&subcollection_path);
        fs::create_dir_all(&subcollection_dir).unwrap();

        let mut collection_mut = Collection::new(subcollection_path);

        if let Some(title) = title {
            collection_mut.title = Some(title.to_string());
        }

        let subcollection = Arc::new(collection_mut);

        let subcollection_moving = Arc::clone(&subcollection);
        let mutation = |collection: &mut Collection| {
            collection.subcollections.push(subcollection_moving);
        };

        if context.update_collection(mutation, &path.path).is_err() {
            return Either::Right(not_found(&context, &language, &path.path));
        }

        subcollection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", subcollection.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    };

    if slug_trimmed.is_empty() {
        let slug = slugify(title_trimmed);
        let mut subcollection_path = if path.path.is_empty() {
            slug.to_string()
        } else {
            format!("{}/{slug}", &path.path)
        };

        let mut duplicate = 1;
        while context.site_dir.join(&subcollection_path).exists() {
            subcollection_path = if path.path.is_empty() {
                format!("{slug}_{duplicate}")
            } else {
                format!("{}/{slug}_{duplicate}", &path.path)
            };
            duplicate += 1;
        }

        succeed(subcollection_path, Some(title_trimmed))
    } else {
        if let Err(message) = validate_slug(slug_trimmed) {
            return error(Some(&message), None);
        }

        let subcollection_path = if path.path.is_empty() {
            slug_trimmed.to_string()
        } else {
            format!("{}/{slug_trimmed}", &path.path)
        };

        if context.site_dir.join(&subcollection_path).exists() {
            let message = language.translations.permalink_already_used_by_another_collection_playlist_or_video;
            error(Some(message), None)
        } else {
            let title = if title_trimmed.is_empty() { None } else { Some(title_trimmed) };
            succeed(subcollection_path, title)
        }
    }
}

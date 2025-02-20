// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Collection, Context, ContextPath, Field, VideoOrder};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::CollectionFormFeedback;

#[derive(Deserialize)]
pub struct UpdateForm {
    description: String,
    download: String,
    embedding: String,
    offline: Option<String>,
    title: String,
    unlisted: Option<String>,
    video_order: String
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let description_trimmed = form.description.trim();

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

    let offline = form.offline.is_some();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();
    let video_order = VideoOrder::from_key(&form.video_order).unwrap();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |collection_mut: &mut Collection| {
            collection_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            collection_mut.download = download;
            collection_mut.embedding = embedding;
            collection_mut.offline = offline;
            collection_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            collection_mut.unlisted = unlisted;
            collection_mut.video_order = video_order;
        };

        let collection = match context.update_collection(mutation, &path.path) {
            Ok(collection) => collection,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        collection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::Update {
            description: description_trimmed,
            download,
            embedding,
            offline,
            title: Field::valid(title_trimmed),
            unlisted,
            video_order: &form.video_order
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &language))
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    Field,
    PlatformIntegration,
    SitePath,
    VideoOrder
};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::CollectionFormFeedback;

#[derive(Deserialize)]
pub struct UpdateForm {
    copy_link: String,
    description: String,
    download: String,
    embedding: String,
    feeds: String,
    offline: Option<String>,
    platform_integration: String,
    title: String,
    unlisted: Option<String>,
    video_order: String
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let copy_link = match form.copy_link.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

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

    let feeds = match form.feeds.as_str() {
        "disabled" => Some(false),
        "enabled" => Some(true),
        _ => None
    };

    let platform_integration = match form.platform_integration.as_str() {
        "" => None,
        other => Some(PlatformIntegration::from_key(other).unwrap())
    };

    let offline = form.offline.is_some();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();
    let video_order = VideoOrder::from_key(&form.video_order).unwrap();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |collection_mut: &mut Collection| {
            collection_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            collection_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            collection_mut.video_order = video_order;

            let mut must_propagate = false;

            must_propagate |= collection_mut.copy_link_local_set(copy_link);
            must_propagate |= collection_mut.download_local_set(download);
            must_propagate |= collection_mut.embedding_local_set(embedding);
            must_propagate |= collection_mut.feeds_local_set(feeds);
            must_propagate |= collection_mut.offline_local_set(offline);
            must_propagate |= collection_mut.platform_integration_local_set(platform_integration);
            must_propagate |= collection_mut.unlisted_local_set(unlisted);

            if must_propagate {
                collection_mut.propagate_options();
            }
        };

        match context.update_collection(mutation, &site_path) {
            CollectionUpdate::Updated { collection_updated, .. } => {
                collection_updated.write_manifest(&context);
                let redirect_url = format!("/collection/{}", &collection_updated.site_path.normalized());
                Either::Left(Redirect::to(redirect_url).see_other())
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &site_path))
            }
        }
    } else {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = CollectionFormFeedback::Update {
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            feeds,
            offline,
            platform_integration,
            title: Field::valid(title_trimmed),
            unlisted,
            video_order: &form.video_order
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &language))
    }
}

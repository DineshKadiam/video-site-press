// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    Collection,
    CollectionUpdate,
    ContainerOrder,
    PlatformIntegration,
    SitePath,
    VideoOrder
};

use crate::{Context, Field, Session};
use crate::routes;
use crate::widgets::not_found;

use super::edit::edit_page;
use super::edit::CollectionFormFeedback;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct UpdateForm {
    container_order: String,
    copy_link: String,
    description: String,
    download: String,
    embedding: String,
    feeds: String,
    offline: Option<String>,
    platform_integration: String,
    sort_number: String,
    title: String,
    unlisted: Option<String>,
    video_order: String
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);

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

    let sort_number_trimmed = form.sort_number.trim();
    let sort_number = match sort_number_trimmed.is_empty() {
        true => None,
        false => Some(
            sort_number_trimmed.parse::<i32>().map_err(|err| err.to_string())
        )
    };

    let container_order = ContainerOrder::from_key(&form.container_order).unwrap();
    let offline = form.offline.is_some();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();
    let video_order = VideoOrder::from_key(&form.video_order).unwrap();

    if sort_number.as_ref().is_some_and(|sort_number| sort_number.is_err()) {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &session, &site_path))
        };

        let form_feedback = CollectionFormFeedback::Update {
            container_order: &form.container_order,
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            feeds,
            offline,
            platform_integration,
            sort_number: Field::validated(sort_number_trimmed, sort_number.and_then(|sort_number| sort_number.err())),
            title: Field::valid(title_trimmed),
            unlisted,
            video_order: &form.video_order
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &session))
    } else {
        let mutation = |collection_mut: &mut Collection| {
            collection_mut.container_order = container_order;
            collection_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            collection_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
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
            CollectionUpdate::InvalidManifest { collection } => {
                Either::Right(pending_manifest_errors(&collection, &context, &session))
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &session, &site_path))
            }
            CollectionUpdate::Updated { collection_updated, .. } => {
                collection_updated.write_manifest(&context);
                let redirect_url = routes::collection_edit(&collection_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    }
}

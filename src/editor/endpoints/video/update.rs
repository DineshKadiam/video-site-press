// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use chrono::NaiveDate;
use serde_derive::Deserialize;

use crate::{
    Context,
    Field,
    PlatformIntegration,
    SitePath,
    Video
};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::VideoFormFeedback;

#[derive(Deserialize)]
pub struct VideoUpdateForm {
    copy_link: String,
    description: String,
    download: String,
    embedding: String,
    offline: Option<String>,
    platform_integration: String,
    release_date: String,
    sort_number: String,
    title: String,
    unlisted: Option<String>
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<VideoUpdateForm>,
    site_path: ActixPath<SitePath>
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

    let offline = form.offline.is_some();

    let platform_integration = match form.platform_integration.as_str() {
        "" => None,
        other => Some(PlatformIntegration::from_key(other).unwrap())
    };

    let release_date_trimmed = form.release_date.trim();
    let release_date = if release_date_trimmed.is_empty() {
        None
    } else {
        Some(
            NaiveDate::parse_from_str(release_date_trimmed, "%Y-%m-%d")
                .map_err(|err| err.to_string())
        )
    };

    let sort_number_trimmed = form.sort_number.trim();
    let sort_number = if sort_number_trimmed.is_empty() {
        None
    } else {
        Some(
            sort_number_trimmed.parse::<i32>().map_err(|err| err.to_string())
        )
    };

    let title_trimmed = form.title.trim();

    let unlisted = form.unlisted.is_some();

    if release_date.as_ref().is_some_and(|release_date| release_date.is_err()) ||
        sort_number.as_ref().is_some_and(|sort_number| sort_number.is_err()) {
        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = VideoFormFeedback::Update {
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            offline,
            platform_integration,
            release_date: Field::validated(release_date_trimmed, release_date.and_then(|release_date| release_date.err())),
            sort_number: Field::validated(sort_number_trimmed, sort_number.and_then(|sort_number| sort_number.err())),
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &video)
        )
    } else {
        let mutation = |video_mut: &mut Video| {
            video_mut.copy_link_local_set(copy_link);
            video_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video_mut.download_local_set(download);
            video_mut.embedding_local_set(embedding);
            video_mut.offline_local_set(offline);
            video_mut.platform_integration_local_set(platform_integration);
            video_mut.release_date = release_date.and_then(|release_date| release_date.ok());
            video_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
            video_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video_mut.unlisted_local_set(unlisted);
            video_mut.write_manifest(&context);
        };

        let video = match context.update_video(mutation, &site_path) {
            Ok(video) => video,
            Err(()) => return Either::Right(not_found(&context, &language, &site_path))
        };

        let redirect_url = format!("/video/{}", video.site_path.normalized());
        Either::Left(Redirect::to(redirect_url).see_other())
    }
}

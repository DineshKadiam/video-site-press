// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use chrono::NaiveDate;
use serde_derive::Deserialize;

use crate::{Context, ContextPath, Field, Video};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::VideoFormFeedback;

#[derive(Deserialize)]
pub struct VideoUpdateForm {
    description: String,
    download: String,
    embedding: String,
    offline: Option<String>,
    release_date: String,
    sort_number: String,
    title: String,
    unlisted: Option<String>
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<VideoUpdateForm>,
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

    let release_date_trimmed = form.release_date.trim();
    let release_date = if release_date_trimmed.is_empty() {
        None
    } else {
        Some(
            NaiveDate::parse_from_str(&release_date_trimmed, "%Y-%m-%d")
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
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = VideoFormFeedback::Update {
            description: description_trimmed,
            download,
            embedding,
            offline,
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
            video_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video_mut.download = download;
            video_mut.embedding = embedding;
            video_mut.offline = offline;
            video_mut.release_date = release_date.and_then(|release_date| release_date.ok());
            video_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
            video_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video_mut.unlisted = unlisted;
            video_mut.write_manifest(&context);
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    }
}

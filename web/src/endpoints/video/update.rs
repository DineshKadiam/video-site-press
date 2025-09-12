// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use chrono::NaiveDate;
use serde_derive::Deserialize;

use hyper8_core::{
    InfallibleResult,
    PlatformIntegration,
    SitePath,
    Video,
    VideoUpdate
};
use hyper8_core::infallible_mutation;
use hyper8_core::MAX_SYNOPSIS_CHARS;

use crate::{Context, Field, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::edit::edit_page;
use super::edit::UpdateFormFeedback;
use super::pending_manifest_errors;

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
    synopsis: String,
    title: String,
    unlisted: Option<String>
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<VideoUpdateForm>,
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

    let synopsis_trimmed = form.synopsis.trim();
    let synopsis = if synopsis_trimmed.is_empty() {
        None
    } else if synopsis_trimmed.chars().count() <= MAX_SYNOPSIS_CHARS {
        Some(Ok(synopsis_trimmed.to_string()))
    } else {
        Some(Err(session_state.language.translations.xxx_characters_maximum(MAX_SYNOPSIS_CHARS)))
    };

    let title_trimmed = form.title.trim();

    let unlisted = form.unlisted.is_some();

    if release_date.as_ref().is_some_and(|release_date| release_date.is_err()) ||
        sort_number.as_ref().is_some_and(|sort_number| sort_number.is_err()) ||
        synopsis.as_ref().is_some_and(|synopsis| synopsis.is_err()) {
        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &session_state, &site_path))
        };

        let form_feedback = UpdateFormFeedback {
            copy_link,
            description: description_trimmed,
            download,
            embedding,
            offline,
            platform_integration,
            release_date: Field::validated(release_date_trimmed, release_date.and_then(|release_date| release_date.err())),
            sort_number: Field::validated(sort_number_trimmed, sort_number.and_then(|sort_number| sort_number.err())),
            synopsis: Field::validated(synopsis_trimmed, synopsis.and_then(|synopsis| synopsis.err())),
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &session_state, &video)
        )
    } else {
        let mutation = |video_mut: &mut Video| -> InfallibleResult {
            video_mut.copy_link_local_set(copy_link);
            video_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video_mut.download_local_set(download);
            video_mut.embedding_local_set(embedding);
            video_mut.offline_local_set(offline);
            video_mut.platform_integration_local_set(platform_integration);
            video_mut.release_date = release_date.and_then(|release_date| release_date.ok());
            video_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
            video_mut.synopsis = synopsis.and_then(|synopsis| synopsis.ok());
            video_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video_mut.unlisted_local_set(unlisted);
            video_mut.write_manifest(&context);

            Ok(())
        };

        match context.update_video(mutation, &site_path) {
            VideoUpdate::Error(_) => infallible_mutation!(),
            VideoUpdate::InvalidManifest { video } => {
                Either::Right(pending_manifest_errors(&context, &session_state, &video))
            }
            VideoUpdate::NotFound => {
                Either::Right(not_found(&context, &session_state, &site_path))
            }
            VideoUpdate::Success { video_updated, .. } => {
                let redirect_url = routes::video_edit(&video_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    }
}

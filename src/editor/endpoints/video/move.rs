// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Context,
    Field,
    SitePath,
    Video,
    VideoUpdate
};
use crate::editor::routes;
use crate::editor::widgets::{error_page, not_found};
use crate::util::validate_slug;

use super::dangerous_actions::dangerous_actions_page;
use super::dangerous_actions::MoveFormFeedback;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct VideoMoveForm {
    slug: String
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<VideoMoveForm>,
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
        Some(String::from("This permalink is already used by another collection, playlist or video."))
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |video: &mut Video| {
            match fs::rename(
                context.site_dir.join(video.site_path.filesystem_relative()),
                context.site_dir.join(new_path.filesystem_relative())
            ) {
                Ok(()) => {
                    video.site_path = new_path;
                    Ok(())
                }
                Err(err) => Err(err.to_string())
            }
        };

        match context.update_video(mutation, &site_path) {
            VideoUpdate::Error(err) => {
                let language = context.get_editor_language();
                Either::Right(error_page(&context, &language, &err))
            }
            VideoUpdate::InvalidManifest { video } => {
                Either::Right(pending_manifest_errors(&context, &language, &video))
            }
            VideoUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &site_path))
            }
            VideoUpdate::Success { video_updated, .. } => {
                let redirect_url = routes::video_dangerous_actions(&video_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    } else {
        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = MoveFormFeedback {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            dangerous_actions_page(&context, Some(form_feedback), &language, &video)
        )
    }
}

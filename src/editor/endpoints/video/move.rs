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
    Video
};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::VideoFormFeedback;

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
            fs::rename(
                context.site_dir.join(video.site_path.filesystem_relative()),
                context.site_dir.join(new_path.filesystem_relative())
            ).unwrap();

            video.site_path = new_path;
        };

        let video = match context.update_video(mutation, &site_path) {
            Ok(video) => video,
            Err(()) => return Either::Right(not_found(&context, &language, &site_path))
        };

        let redirect_url = format!("/video/{}", video.site_path.normalized());
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = VideoFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &video)
        )
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use language_tags::LanguageTag;
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

use super::edit_subtitles::edit_subtitles_page;
use super::edit_subtitles::UpdateFormFeedback;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct SubtitlesUpdateForm {
    content: String,
    label: String,
    language: String
}

pub async fn update_subtitles(
    context: Data<Arc<Context>>,
    form: Form<SubtitlesUpdateForm>,
    path: ActixPath<(String, String)>
) -> Either<Redirect, HttpResponse> {
    let (file_name, site_path_str) = path.into_inner();
    let site_path = SitePath::new(site_path_str);

    let content = &form.content;
    let label_trimmed = form.label.trim();
    let language_trimmed = form.language.trim();

    let label_error = match label_trimmed.is_empty() {
        // TODO: Translate
        true => Some("Label can not be empty"),
        false => None
    };

    let language = match language_trimmed.is_empty() {
        true => Ok(None),
        false => match LanguageTag::parse(language_trimmed) {
            Ok(language_tag) => {
                match language_tag.validate() {
                    Ok(()) => Ok(Some(language_trimmed.to_string())),
                    Err(err) => Err(err.to_string())
                }
            }
            Err(err) => Err(err.to_string())
        }
    };

    if label_error.is_some() || language.is_err() {
        let editor_language = context.get_editor_language();

        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &editor_language, &site_path))
        };

        let form_feedback = UpdateFormFeedback {
            content,
            label: Field::validated(label_trimmed, label_error),
            language: Field::validated(language_trimmed, language.err())
        };

        Either::Right(
            edit_subtitles_page(&context, &file_name, Some(form_feedback), &editor_language, &video)
        )
    } else {
        let mutation = |video_mut: &mut Video| {
            match video_mut.subtitles
                .iter_mut()
                .find(|subtitle_file| subtitle_file.file_name == *file_name) {
                Some(subtitle_file) => {
                    subtitle_file.explicitly_defined = true;
                    subtitle_file.label = label_trimmed.to_string();
                    subtitle_file.language = language.unwrap();

                    let subtitle_file_path = context.site_dir
                        .join(video_mut.site_path.filesystem_relative())
                        .join(&subtitle_file.file_name);

                    fs::write(subtitle_file_path, content).unwrap();

                    video_mut.write_manifest(&context);

                    Ok(())
                }
                None => {
                    let editor_language = context.get_editor_language();
                    Err(editor_language.translations.the_subtitles_to_be_updated_were_not_found.to_string())
                }
            }
        };

        match context.update_video(mutation, &site_path) {
            VideoUpdate::Error(err) => {
                let editor_language = context.get_editor_language();
                Either::Right(error_page(&context, &editor_language, &err))
            }
            VideoUpdate::NotFound => {
                let editor_language = context.get_editor_language();
                Either::Right(not_found(&context, &editor_language, &site_path))
            }
            VideoUpdate::PendingErrors { video } => {
                let editor_language = context.get_editor_language();
                Either::Right(pending_manifest_errors(&context, &editor_language, &video))
            }
            VideoUpdate::Success { video_updated, .. } => {
                let redirect_url = routes::video_subtitles(&video_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    }
}

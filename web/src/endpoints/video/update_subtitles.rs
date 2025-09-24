// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use language_tags::LanguageTag;
use serde_derive::Deserialize;

use hyper8_core::{
    SitePath,
    SubtitleFile,
    Video,
    VideoUpdate
};

use crate::{Context, Field, SessionState};
use crate::routes;
use crate::widgets::{error_page, not_found};

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
    path: ActixPath<(String, String)>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    let (file_name, site_path_str) = path.into_inner();
    let site_path = SitePath::new(site_path_str);

    let content = &form.content;
    let label_trimmed = form.label.trim();
    let language_trimmed = form.language.trim();

    let label_error = match label_trimmed.is_empty() {
        true => {
            let session_language = SessionState::language(&context.server_mode, &session);
            Some(session_language.translations.label_can_not_be_empty)
        }
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
        let session_state = SessionState::all(&context.server_mode, &session);

        let video = match context.get_video(&site_path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &session_state, &site_path))
        };

        let form_feedback = UpdateFormFeedback {
            content,
            label: Field::validated(label_trimmed, label_error),
            language: Field::validated(language_trimmed, language.err())
        };

        Either::Right(
            edit_subtitles_page(&context, &file_name, Some(form_feedback), &session_state, &video)
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

                    fs::write(&subtitle_file_path, content).unwrap();

                    // TODO: Optimize by not reading the content back in from the file,
                    //       instead using the in-memory content variable (needs some
                    //       light refactoring).
                    subtitle_file.subtitles = SubtitleFile::parse_subtitles(
                        &subtitle_file.file_name,
                        &subtitle_file_path
                    );

                    video_mut.write_manifest(&context);

                    Ok(())
                }
                None => {
                    let session_language = SessionState::language(&context.server_mode, &session);
                    Err(session_language.translations.the_subtitles_to_be_updated_were_not_found.to_string())
                }
            }
        };

        match context.update_video(mutation, &site_path) {
            VideoUpdate::Error(err) => {
                let session_state = SessionState::all(&context.server_mode, &session);
                Either::Right(error_page(&context, &err, &session_state))
            }
            VideoUpdate::InvalidManifest { video } => {
                let session_state = SessionState::all(&context.server_mode, &session);
                Either::Right(pending_manifest_errors(&context, &session_state, &video))
            }
            VideoUpdate::NotFound => {
                let session_state = SessionState::all(&context.server_mode, &session);
                Either::Right(not_found(&context, &session_state, &site_path))
            }
            VideoUpdate::Success { video_updated, .. } => {
                let redirect_url = routes::video_subtitles(&video_updated);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    }
}

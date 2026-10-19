// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    Chapter,
    InfallibleResult,
    SitePath,
    Timecode,
    Video,
    VideoUpdate
};
use hyper8_core::infallible_mutation;

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct ChapterData {
    begin: String,
    end: Option<String>,
    title: String
}

#[derive(Deserialize)]
pub struct ChaptersUpdateForm {
    data: String
}

pub async fn update_chapters(
    context: Data<Arc<Context>>,
    form: Form<ChaptersUpdateForm>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let data = &form.data;

    let chapters = serde_json::from_str::<Vec<ChapterData>>(data)
        .unwrap()
        .into_iter()
        .map(|chapter| {
            let begin = Timecode::try_from(chapter.begin.as_str()).unwrap();
            let end = chapter.end.map(|end| Timecode::try_from(end.as_str()).unwrap());
            let title = chapter.title;

            Chapter {
                begin,
                end,
                title
            }
        })
        .collect::<Vec<Chapter>>();

    let chapters = if chapters.is_empty() {
        None
    } else {
        Some(chapters)
    };

    let mutation = |video_mut: &mut Video| -> InfallibleResult {
        video_mut.chapters = chapters;
        video_mut.write_manifest(&context);
        Ok(())
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(_) => infallible_mutation!(),
        VideoUpdate::InvalidManifest { video } => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(pending_manifest_errors(&context, &session_state, &video))
        }
        VideoUpdate::NotFound => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_chapters(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

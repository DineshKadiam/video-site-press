// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, SitePath, Video, VideoUpdate};
use crate::editor::routes;
use crate::editor::widgets::{error_page, not_found};

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct DeleteVersionForm {
    file_name: String
}

pub async fn delete_version(
    context: Data<Arc<Context>>,
    form: Form<DeleteVersionForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(version) = video_mut.versions
            .iter()
            .position(|version| version.file_name == form.file_name)
            .map(|index| video_mut.versions.remove(index)) {
            let version_path = context.site_dir
                .join(site_path.filesystem_relative())
                .join(&version.file_name);

            video_mut.write_manifest(&context);
            let _ = fs::remove_file(&version_path);

            Ok(())
        } else {
            let language = context.get_editor_language();
            Err(language.translations.the_version_requested_for_deletion_was_not_found.to_string())
        }
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(err) => {
            let language = context.get_editor_language();
            Either::Right(error_page(&context, &language, &err))
        }
        VideoUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
        VideoUpdate::PendingErrors { video } => {
            let language = context.get_editor_language();
            Either::Right(pending_manifest_errors(&context, &language, &video))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_media(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath};
use crate::editor::widgets::error_page;

pub async fn to_collection(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Redirect {
    context.convert_to_collection(&site_path);
    let redirect_url = format!("/collection/{}", site_path.normalized());
    Redirect::to(redirect_url).see_other()
}

pub async fn to_playlist(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.convert_to_playlist(&site_path) {
        Ok(()) => {
            let redirect_url = format!("/playlist/{}", site_path.normalized());
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            let language = context.get_editor_language();
            // TODO: Return error in context of originating page (collection/video/empty)
            Either::Right(error_page(&context, &language, &err))
        }
    }
}

pub async fn to_video(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.convert_to_video(&site_path) {
        Ok(()) => {
            let redirect_url = format!("/video/{}", site_path.normalized());
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            let language = context.get_editor_language();
            // TODO: Return error in context of originating page (collection/playlist/empty)
            Either::Right(error_page(&context, &language, &err))
        }
    }
}

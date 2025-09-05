// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::SitePath;

use crate::{Context, Session};
use crate::routes;
use crate::widgets::error_page;

pub async fn to_collection(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);

    match context.convert_to_collection(&session.language, &site_path) {
        Ok(collection) => {
            let redirect_url = routes::collection_edit(&collection);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            // TODO: Return error in context of originating page (playlist/video/empty)
            Either::Right(error_page(&context, &err, &session))
        }
    }
}

pub async fn to_playlist(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);

    match context.convert_to_playlist(&session.language, &site_path) {
        Ok(playlist) => {
            let redirect_url = routes::playlist_edit(&playlist);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            // TODO: Return error in context of originating page (collection/video/empty)
            Either::Right(error_page(&context, &err, &session))
        }
    }
}

pub async fn to_video(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);

    match context.convert_to_video(&session.language, &site_path) {
        Ok(video) => {
            let redirect_url = routes::video_edit(&video);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            // TODO: Return error in context of originating page (collection/playlist/empty)
            Either::Right(error_page(&context, &err, &session))
        }
    }
}

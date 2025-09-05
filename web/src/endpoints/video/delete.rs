// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::{SitePath, VideoParent};

use crate::{Context, Session};
use crate::routes;
use crate::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.delete_video(&site_path) {
        Ok(Some(parent)) => {
            let redirect_url = match parent {
                VideoParent::Collection(collection) => routes::collection_edit(&collection),
                VideoParent::Playlist(playlist) => routes::playlist_edit(&playlist)
            };
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Ok(None) => Either::Left(Redirect::to(routes::DASHBOARD).see_other()),
        Err(()) => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
    }
}

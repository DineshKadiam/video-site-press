// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::SitePath;

use crate::{Context, Session};
use crate::routes;
use crate::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.delete_playlist(&site_path) {
        Ok(Some(parent_collection)) => {
            let redirect_url = routes::collection_edit(&parent_collection);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Ok(None) => Either::Left(Redirect::to(routes::DASHBOARD).see_other()),
        Err(()) => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
    }
}

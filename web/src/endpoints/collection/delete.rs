// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::SitePath;

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.delete_collection(&site_path) {
        Ok(Some(parent_collection)) => {
            let redirect_url = routes::collection_edit(&parent_collection);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Ok(None) => Either::Left(Redirect::to(routes::DASHBOARD).see_other()),
        Err(()) => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
    }
}

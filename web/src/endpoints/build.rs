// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Path as ActixPath, Redirect};

use hyper8_core::SitePath;
use hyper8_core::perform_build;

use crate::{Context, SessionState};
use crate::widgets::error_page;

pub async fn build(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let syncing = context.sync_state.lock().unwrap().is_some();

    if syncing {
        let session_state = SessionState::all(&context.server_mode, &session);
        let message = session_state.language.translations.preview_is_only_available_again_after_the_current_sync_is_finished;
        return Either::Right(error_page(&context, message, &session_state));
    }

    if let Err(err) = perform_build(&context) {
        let session_state = SessionState::all(&context.server_mode, &session);
        let t_build_failed = session_state.language.translations.build_failed;
        let message = format!("{t_build_failed}\n{err}");
        return Either::Right(error_page(&context, &message, &session_state));
    }

    let redirect_url = format!("/preview/{}", site_path.normalized());
    Either::Left(Redirect::to(redirect_url).see_other())
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Path as ActixPath, Redirect};

use hyper8_core::SitePath;
use hyper8_core::perform_build;

use crate::{Context, Session};
use crate::widgets::error_page;

pub async fn build(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let pending_deployment = context.deploying.lock().unwrap().active();

    if pending_deployment {
        let session = Session::new(&session);
        let message = session.language.translations.preview_is_only_available_again_after_the_current_deployment_is_finished;
        return Either::Right(error_page(&context, message, &session));
    }

    perform_build(&context);

    let redirect_url = format!("/preview/{}", site_path.normalized());
    Either::Left(Redirect::to(redirect_url).see_other())
}

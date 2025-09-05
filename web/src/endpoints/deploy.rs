// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Redirect};

use hyper8_core::{config_probe, perform_build, start_deploy};

use crate::{Context, Session};
use crate::routes;
use crate::widgets::error_page;

pub async fn deploy(
    context: Data<Arc<Context>>,
    session: ActixSession
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);
    let translations = &session.language.translations;
    let site = context.get_site();

    if site.base_url.is_none() {
        let site_edit_url = routes::SITE_EDIT;
        let message = translations.public_address_must_be_configured_for_deployment_text(site_edit_url);
        return Either::Right(error_page(&context, &message, &session));
    }

    if !site.complete_deployment_config() {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy;
        return Either::Right(error_page(&context, message, &session));
    }

    if let Err(err) = config_probe(&context, &site, translations) {
        let t_deployment_configuration_could_not_be_verified = translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} {err}");
        return Either::Right(error_page(&context, &message, &session));
    }

    perform_build(&context);

    match start_deploy(&context.core_context, &site, translations) {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &err, &session))
    }
}

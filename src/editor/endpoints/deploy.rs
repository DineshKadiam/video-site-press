// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Redirect};

use crate::Context;
use crate::build::perform_build;
use crate::deploy::{config_probe, start_deploy};
use crate::editor::routes;
use crate::editor::widgets::error_page;

pub async fn deploy(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();
    let translations = &language.translations;
    let site = context.get_site();

    if site.base_url.is_none() {
        let site_edit_url = routes::SITE_EDIT;
        let message = translations.public_address_must_be_configured_for_deployment_text(site_edit_url);
        return Either::Right(error_page(&context, &language, &message));
    }

    if !site.complete_deployment_config() {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy;
        return Either::Right(error_page(&context, &language, message));
    }

    if let Err(err) = config_probe(&context, &site, translations) {
        let t_deployment_configuration_could_not_be_verified = language.translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} {err}");
        return Either::Right(error_page(&context, &language, &message));
    }

    perform_build(&context);

    match start_deploy(&context, &site, translations) {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &language, &err))
    }
}

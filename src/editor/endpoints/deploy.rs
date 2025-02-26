// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Redirect};

use crate::{Context, SiteContent};
use crate::build::perform_build;
use crate::deploy::{config_probe, perform_deploy};
use crate::editor::widgets::error_page;

pub async fn configure(context: Data<Arc<Context>>) -> Redirect {
    let site = context.get_site();

    let redirect_url = match site.content {
        SiteContent::Collection(_) => "/collection/#edit-site",
        SiteContent::Playlist(_) => "/playlist/#edit-site",
        SiteContent::Video(_) => "/video/#edit-site",
        // TODO: Is there any point to implementing a site form for an uninitialized site?
        //       (at this point offering a link to configure deployment, when there is no
        //       way to configure anything, is anyhow definitely wrong).
        _ => "/"
    };

    Redirect::to(redirect_url).see_other()
}

pub async fn deploy(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();
    let site = context.get_site();

    if site.base_url.is_none() {
        // TODO: Localize
        let message = "Base url is not set, therefore the deploy config cannot be probed, therefore deploying is not permitted for safety reasons.";
        return Either::Right(error_page(&context, &language, message));
    }

    if !site.deploy_config.complete() {
        let t_deployment_configuration_is_incomplete_can_not_deploy = language.translations.deployment_configuration_is_incomplete_can_not_deploy;
        return Either::Right(error_page(&context, &language, t_deployment_configuration_is_incomplete_can_not_deploy));
    }

    if let Err(err) = config_probe(&context, &site) {
        let t_deployment_configuration_could_not_be_verified = language.translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} {err}");
        return Either::Right(error_page(&context, &language, &message));
    }

    perform_build(&context);

    match perform_deploy(&context, &site) {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &language, &err))
    }
}

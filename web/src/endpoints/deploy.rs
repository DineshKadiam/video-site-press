// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Redirect};

use hyper8_core::DeploymentMode;
use hyper8_core::{
    perform_build,
    start_ftp_deployment,
    start_ssh_deployment,
    verify_ftp_deployment_config,
    verify_ssh_deployment_config
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::error_page;

pub async fn deploy_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    deploy(context, DeploymentMode::Ftp, session)
}

pub async fn deploy_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    deploy(context, DeploymentMode::Ssh, session)
}

fn deploy(
    context: Data<Arc<Context>>,
    mode: DeploymentMode,
    session: Session
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    if site.base_url.is_none() {
        let site_edit_url = routes::SITE_EDIT;
        let message = translations.public_address_must_be_configured_for_deployment_text(site_edit_url);
        return Either::Right(error_page(&context, &message, &session_state));
    }

    if (mode.ftp() && !site.ftp_config.complete()) ||
        (mode.ssh() && !site.ssh_config.complete())
    {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy;
        return Either::Right(error_page(&context, message, &session_state));
    }

    let verification = if mode.ftp() {
        verify_ftp_deployment_config(&site, translations)
    } else {
        verify_ssh_deployment_config(&site, translations)
    };

    if let Err(err) = verification {
        let t_deployment_configuration_could_not_be_verified = translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} {err}");
        return Either::Right(error_page(&context, &message, &session_state));
    }

    perform_build(&context);

    let deploy_result = if mode.ftp() {
        start_ftp_deployment(
            &context.core_context,
            mode.simulate(),
            &site,
            translations
        )
    } else {
        start_ssh_deployment(
            &context.core_context,
            mode.simulate(),
            &site,
            translations
        )
    };

    match deploy_result {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &err, &session_state))
    }
}

pub async fn simulate_deploy_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    deploy(context, DeploymentMode::FtpSimulated, session)
}

pub async fn simulate_deploy_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    deploy(context, DeploymentMode::SshSimulated, session)
}

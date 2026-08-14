// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Either, Redirect};

use hyper8_core::{DeploymentProtocol, Site};
use hyper8_core::{
    check_deployment_config,
    perform_build,
    start_deployment
};
use hyper8_translations::Translations;
use hyper8_util::html_escape_outside_attribute;

use crate::{Context, Location, SessionState};
use crate::widgets::{error_page, layout};

fn check(
    context: Data<Arc<Context>>,
    protocol: DeploymentProtocol,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    if let Err(err) = ensure_config(protocol, &site, translations) {
        return error_page(&context, &err, &session_state);
    }

    let steps = check_deployment_config(protocol, &site, translations);

    let failed_icon = hyper8_icons::failure(translations.failed);
    let successful_icon = hyper8_icons::success(translations.successful);

    let body = steps
        .iter()
        .map(|step| {
            match step {
                Ok(message) => {
                    let message_escaped = html_escape_outside_attribute(message);
                    format!("<div>{successful_icon} {message_escaped}</div>")
                }
                Err(message) => {
                    let message_escaped = html_escape_outside_attribute(message);
                    format!("<div>{failed_icon} {message_escaped}</div>")
                }
            }
        })
        .collect::<Vec<_>>()
        .join("");

    let html = layout(
        &body,
        &context,
        Location::AuxiliaryPage,
        &session_state,
        &site,
        translations.deployment_configuration_check
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn check_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    check(context, DeploymentProtocol::Ftp, session)
}

pub async fn check_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    check(context, DeploymentProtocol::Ssh, session)
}

fn ensure_config<'a>(
    protocol: DeploymentProtocol,
    site: &Site,
    translations: &'a Translations
) -> Result<(), &'a str> {
    if site.complete_deployment_config(protocol) {
        Ok(())
    } else {
        Err(translations.deployment_configuration_is_incomplete_can_not_deploy)
    }
}

fn execute(
    context: Data<Arc<Context>>,
    protocol: DeploymentProtocol,
    session: Session,
    simulate: bool
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    if let Err(err) = ensure_config(protocol, &site, translations) {
        return Either::Right(error_page(&context, &err, &session_state));
    }

    perform_build(&context);

    let deploy_result = start_deployment(
        &context.core_context,
        protocol,
        simulate,
        &site,
        translations
    );

    match deploy_result {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &err, &session_state))
    }
}

pub async fn execute_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    execute(
        context,
        DeploymentProtocol::Ftp,
        session,
        false
    )
}

pub async fn execute_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    execute(
        context,
        DeploymentProtocol::Ssh,
        session,
        false
    )
}

pub async fn simulate_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    execute(
        context,
        DeploymentProtocol::Ftp,
        session,
        true
    )
}

pub async fn simulate_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    execute(
        context,
        DeploymentProtocol::Ssh,
        session,
        true
    )
}

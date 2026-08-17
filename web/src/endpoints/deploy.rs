// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Either, Redirect};

use hyper8_core::{Site, SyncProtocol};
use hyper8_core::{
    check_deployment_config,
    perform_build,
    start_sync
};
use hyper8_translations::Translations;
use hyper8_util::html_escape_outside_attribute;

use crate::{Context, Location, SessionState};
use crate::widgets::{error_page, layout};

fn check(
    context: Data<Arc<Context>>,
    protocol: SyncProtocol,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    if let Err(err) = ensure_config(protocol, &site, translations) {
        return error_page(&context, &err, &session_state);
    }

    let steps = check_deployment_config(protocol, &site, translations);

    let r_steps = steps
        .iter()
        .map(|step| {
            let context = if let Some(context) = &step.context {
                let context_escaped = html_escape_outside_attribute(context);
                format!(r#"<span class="check_context">{context_escaped}</span>"#)
            } else {
                String::new()
            };

            let icon = if step.failed() {
                let icon = hyper8_icons::failure(translations.failed);
                format!(r#"<span style="color: red;">{icon}</span>"#)
            } else if step.passed() {
                let icon = hyper8_icons::success(translations.successful);
                format!(r#"<span style="color: green;">{icon}</span>"#)
            } else /* if step.skipped() */ {
                // TODO: Dedicated icon (e.g. "a slash/strike")
                let icon = hyper8_icons::failure(translations.skipped);
                format!(r#"<span style="color: orange;">{icon}</span>"#)
            };

            let message = if let Some(message) = &step.message {
                let message_escaped = html_escape_outside_attribute(message);
                format!(r#"<p class="check_message">{message_escaped}</p>"#)
            } else {
                String::new()
            };

            let label = &step.label;

            format!(r#"<div>{icon} <span class="check_subject">{label}</span> {context}</div>{message}"#)
        })
        .collect::<Vec<_>>()
        .join("");

    let body = format!(r#"<div class="check">{r_steps}</div>"#);

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
    check(context, SyncProtocol::Ftp, session)
}

pub async fn check_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    check(context, SyncProtocol::Ssh, session)
}

fn ensure_config<'a>(
    protocol: SyncProtocol,
    site: &Site,
    translations: &'a Translations
) -> Result<(), &'a str> {
    if site.complete_deployment_config(protocol) {
        Ok(())
    } else {
        Err(translations.deployment_configuration_is_incomplete_can_not_deploy)
    }
}

fn sync(
    context: Data<Arc<Context>>,
    protocol: SyncProtocol,
    session: Session,
    simulate: bool
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    if let Err(err) = ensure_config(protocol, &site, translations) {
        return Either::Right(error_page(&context, &err, &session_state));
    }

    if let Err(err) = perform_build(&context) {
        let t_build_failed = translations.build_failed;
        let message = format!("{t_build_failed}\n{err}");
        return Either::Right(error_page(&context, &message, &session_state));
    }

    let sync_result = start_sync(
        &context.core_context,
        protocol,
        simulate,
        &site,
        translations
    );

    match sync_result {
        Ok(()) => Either::Left(Redirect::to("/deployment").see_other()),
        Err(err) => Either::Right(error_page(&context, &err, &session_state))
    }
}

pub async fn sync_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        SyncProtocol::Ftp,
        session,
        false
    )
}

pub async fn sync_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        SyncProtocol::Ssh,
        session,
        false
    )
}

pub async fn simulate_ftp(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        SyncProtocol::Ftp,
        session,
        true
    )
}

pub async fn simulate_ssh(
    context: Data<Arc<Context>>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        SyncProtocol::Ssh,
        session,
        true
    )
}

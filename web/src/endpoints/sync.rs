// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Either, Form, Json, Redirect};
use indoc::formatdoc;
use serde_derive::{Deserialize, Serialize};

use hyper8_core::{
    FtpConfig,
    Site,
    SshConfig,
    SyncProtocol
};
use hyper8_core::{
    check_sync_config,
    perform_build,
    start_sync
};
use hyper8_translations::Translations;
use hyper8_util::html_escape_outside_attribute;

use crate::{
    Context,
    Layout,
    Location,
    SessionState
};
use crate::widgets::error_page;

/// Returned as (camel-cased) JSON through the /sync/(ftp|ssh)/config endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigJson {
    pub password_prompt: bool,
    pub private_key: Option<String>,
    pub server: Option<String>,
    pub user: Option<String>
}

#[derive(Deserialize)]
pub struct PromptedPasswordForm {
    password: Option<String>
}

impl From<&FtpConfig> for ConfigJson {
    fn from(ftp_config: &FtpConfig) -> ConfigJson {
        let password_prompt = ftp_config.password_prompt();
        let server = ftp_config.server.clone();
        let user = ftp_config.user.clone();

        ConfigJson {
            password_prompt,
            private_key: None,
            server,
            user
        }
    }
}


impl From<&SshConfig> for ConfigJson {
    fn from(ssh_config: &SshConfig) -> ConfigJson {
        let password_prompt = ssh_config.password_prompt();
        let server = ssh_config.server.clone();
        let user = ssh_config.user.clone();

        let private_key = ssh_config.private_key
            .as_ref()
            .map(|private_key| private_key.display().to_string());

        ConfigJson {
            password_prompt,
            private_key,
            server,
            user
        }
    }
}

fn check(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    protocol: SyncProtocol,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    let prompted_password = &form.password;

    if let Err(err) = ensure_config(
        prompted_password.is_some(),
        protocol,
        &site,
        translations
    ) {
        return error_page(&context, &err, &session_state);
    }

    let steps = check_sync_config(
        prompted_password.as_deref(),
        protocol,
        &site,
        translations
    );

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

    let t_configuration_check = translations.configuration_check;
    let body = formatdoc!(r#"
        <h2>{t_configuration_check}</h2>
        <div class="check">{r_steps}</div>
    "#);

    let mut layout = Layout::new();

    layout.wide();

    let html = layout.render(
        &body,
        &context,
        Location::AuxiliaryPage,
        &session_state,
        &site,
        translations.sync_configuration_check
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn check_ftp(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> HttpResponse {
    check(
        context,
        form,
        SyncProtocol::Ftp,
        session
    )
}

pub async fn check_ssh(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> HttpResponse {
    check(
        context,
        form,
        SyncProtocol::Ssh,
        session
    )
}

pub async fn config_ftp(
    context: Data<Arc<Context>>
) -> Json<ConfigJson> {
    let site = context.get_site();
    let config_json = ConfigJson::from(&site.ftp_config);
    Json(config_json)
}

pub async fn config_ssh(
    context: Data<Arc<Context>>
) -> Json<ConfigJson> {
    let site = context.get_site();
    let config_json = ConfigJson::from(&site.ssh_config);
    Json(config_json)
}

fn ensure_config<'a>(
    prompted_password_provided: bool,
    protocol: SyncProtocol,
    site: &Site,
    translations: &'a Translations
) -> Result<(), &'a str> {
    if site.complete_sync_config(protocol) &&
        !(site.prompt_sync_password(protocol) && !prompted_password_provided)
    {
        Ok(())
    } else {
        Err(translations.sync_configuration_is_incomplete_can_not_synchronize)
    }
}

fn sync(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    protocol: SyncProtocol,
    session: Session,
    simulate: bool
) -> Either<Redirect, HttpResponse> {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;
    let site = context.get_site();

    let prompted_password = &form.password;

    if let Err(err) = ensure_config(
        prompted_password.is_some(),
        protocol,
        &site,
        translations
    ) {
        return Either::Right(error_page(&context, &err, &session_state));
    }

    if let Err(err) = perform_build(&context, false) {
        let t_build_failed = translations.build_failed;
        let message = format!("{t_build_failed}\n{err}");
        return Either::Right(error_page(&context, &message, &session_state));
    }

    let sync_result = start_sync(
        &context.core_context,
        prompted_password.as_deref(),
        protocol,
        simulate,
        &site,
        translations
    );

    match sync_result {
        Ok(()) => Either::Left(Redirect::to("/sync_log").see_other()),
        Err(err) => Either::Right(error_page(&context, &err, &session_state))
    }
}

pub async fn sync_ftp(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        form,
        SyncProtocol::Ftp,
        session,
        false
    )
}

pub async fn sync_ssh(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        form,
        SyncProtocol::Ssh,
        session,
        false
    )
}

pub async fn simulate_ftp(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        form,
        SyncProtocol::Ftp,
        session,
        true
    )
}

pub async fn simulate_ssh(
    context: Data<Arc<Context>>,
    form: Form<PromptedPasswordForm>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    sync(
        context,
        form,
        SyncProtocol::Ssh,
        session,
        true
    )
}

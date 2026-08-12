// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    AspectRatio,
    Language,
    Site,
    SiteUpdate,
    SiteUrl,
    Theme
};

use crate::{Context, Field, SessionState};
use crate::routes;

use super::edit::edit_page;
use super::edit::UpdateFormFeedback;
use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct UpdateForm {
    base_url: String,
    clean_urls: Option<String>,
    ftp_config_password: String,
    ftp_config_path: String,
    ftp_config_port: String,
    ftp_config_secure: Option<String>,
    ftp_config_server: String,
    ftp_config_user: String,
    language: String,
    poster_aspect: String,
    ssh_config_password: String,
    ssh_config_path: String,
    ssh_config_port: String,
    ssh_config_server: String,
    ssh_config_user: String,
    theme: String
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    session: Session
) -> Either<Redirect, HttpResponse> {
    let base_url_trimmed = form.base_url.trim();
    let clean_urls = form.clean_urls.is_some();
    let ftp_config_password_trimmed = form.ftp_config_password.trim();
    let ftp_config_path_trimmed = form.ftp_config_path.trim();
    let ftp_config_secure = form.ftp_config_secure.is_some();
    let ftp_config_server_trimmed = form.ftp_config_server.trim();
    let ftp_config_user_trimmed = form.ftp_config_user.trim();
    let language_trimmed = form.language.trim();
    let poster_aspect_trimmed = form.poster_aspect.trim();
    let ssh_config_password_trimmed = form.ssh_config_password.trim();
    let ssh_config_path_trimmed = form.ssh_config_path.trim();
    let ssh_config_server_trimmed = form.ssh_config_server.trim();
    let ssh_config_user_trimmed = form.ssh_config_user.trim();
    let theme = match Theme::from_key(&form.theme) {
        Ok(theme) => theme,
        Err(_) => Theme::default()
    };

    let base_url = if base_url_trimmed.is_empty() {
        None
    } else {
        Some(SiteUrl::parse(base_url_trimmed))
    };

    let poster_aspect = if poster_aspect_trimmed.is_empty() {
        Ok(AspectRatio::Default)
    } else {
        AspectRatio::from_representation(poster_aspect_trimmed)
            .map_err(|err| err.to_string())
    };

    let ftp_config_port_trimmed = form.ftp_config_port.trim();
    let ftp_config_port = if ftp_config_port_trimmed.is_empty() {
        None
    } else {
        Some(
            ftp_config_port_trimmed.parse::<usize>().map_err(|err| err.to_string())
        )
    };

    let ssh_config_port_trimmed = form.ssh_config_port.trim();
    let ssh_config_port = if ssh_config_port_trimmed.is_empty() {
        None
    } else {
        Some(
            ssh_config_port_trimmed.parse::<usize>().map_err(|err| err.to_string())
        )
    };

    if base_url.as_ref().is_some_and(|base_url| base_url.is_err()) ||
        poster_aspect.is_err() ||
        ftp_config_port.as_ref().is_some_and(|ftp_config_port| ftp_config_port.is_err()) ||
        ssh_config_port.as_ref().is_some_and(|ssh_config_port| ssh_config_port.is_err())
    {
        let site = context.get_site();

        let form_feedback = UpdateFormFeedback {
            base_url: Field::validated(base_url_trimmed, base_url.and_then(|base_url| base_url.err().map(|err| err.to_string()))),
            clean_urls,
            ftp_config_password: Field::valid(ftp_config_password_trimmed),
            ftp_config_path: Field::valid(ftp_config_path_trimmed),
            ftp_config_port: Field::validated(ftp_config_port_trimmed, ftp_config_port.and_then(|ftp_config_port| ftp_config_port.err())),
            ftp_config_secure,
            ftp_config_server: Field::valid(ftp_config_server_trimmed),
            ftp_config_user: Field::valid(ftp_config_user_trimmed),
            language: Field::valid(language_trimmed),
            poster_aspect: Field::validated(poster_aspect_trimmed, poster_aspect.err()),
            ssh_config_password: Field::valid(ssh_config_password_trimmed),
            ssh_config_path: Field::valid(ssh_config_path_trimmed),
            ssh_config_port: Field::validated(ssh_config_port_trimmed, ssh_config_port.and_then(|ssh_config_port| ssh_config_port.err())),
            ssh_config_server: Field::valid(ssh_config_server_trimmed),
            ssh_config_user: Field::valid(ssh_config_user_trimmed),
            theme
        };

        let session_state = SessionState::all(&context.server_mode, &session);
        Either::Right(edit_page(&context, Some(form_feedback), &session_state, &site))
    } else {
        fn to_option(trimmed: &str) -> Option<String> {
            match trimmed.is_empty() {
                true => None,
                false => Some(trimmed.to_string())
            }
        }

        let language = if language_trimmed.is_empty() {
            Language::default()
        } else {
            Language::from_code(language_trimmed)
        };

        let mutation = |site_mut: &mut Site| {
            site_mut.base_url = base_url.map(|base_url| base_url.unwrap());
            site_mut.clean_urls = clean_urls;
            site_mut.ftp_config.password = to_option(ftp_config_password_trimmed);
            site_mut.ftp_config.path = to_option(ftp_config_path_trimmed);
            site_mut.ftp_config.port = ftp_config_port.and_then(|ftp_config_port| ftp_config_port.ok());
            site_mut.ftp_config.secure = Some(ftp_config_secure);
            site_mut.ftp_config.server = to_option(ftp_config_server_trimmed);
            site_mut.ftp_config.user = to_option(ftp_config_user_trimmed);
            site_mut.language = language;
            site_mut.poster_aspect = poster_aspect.unwrap();
            site_mut.ssh_config.password = to_option(ssh_config_password_trimmed);
            site_mut.ssh_config.path = to_option(ssh_config_path_trimmed);
            site_mut.ssh_config.port = ssh_config_port.and_then(|ssh_config_port| ssh_config_port.ok());
            site_mut.ssh_config.server = to_option(ssh_config_server_trimmed);
            site_mut.ssh_config.user = to_option(ssh_config_user_trimmed);
            site_mut.theme = theme;
        };

        match context.update_site(mutation) {
            SiteUpdate::InvalidManifest { site } => {
                let session_state = SessionState::all(&context.server_mode, &session);
                Either::Right(pending_manifest_errors(&context, &session_state, &site))
            }
            SiteUpdate::Success { site_updated } => {
                site_updated.write_manifest(&context).unwrap();

                // Banner/Poster assets need recomputation when site.poster_aspect
                // changes (which could have happened here).
                context.compute_job_queue();

                Either::Left(Redirect::to(routes::SITE_EDIT).see_other())
            }
        }
    }
}

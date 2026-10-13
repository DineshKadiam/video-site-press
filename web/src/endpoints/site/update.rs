// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    AspectRatio,
    Degrees,
    Language,
    Percent,
    Site,
    SiteUpdate,
    SiteUrl,
    Theme,
    ThemePaint
};

use crate::{Context, Field, SessionState};
use crate::routes;

use super::edit::edit_page;
use super::edit::UpdateFormFeedback;
use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct UpdateForm {
    dark_theme_accent_brightening_enabled: Option<String>,
    dark_theme_accent_brightening_value: Option<String>,
    dark_theme_accent_chroma_enabled: Option<String>,
    dark_theme_accent_chroma_value: Option<String>,
    dark_theme_accent_hue_enabled: Option<String>,
    dark_theme_accent_hue_value: Option<String>,
    dark_theme_base_chroma_enabled: Option<String>,
    dark_theme_base_chroma_value: Option<String>,
    dark_theme_base_hue_enabled: Option<String>,
    dark_theme_base_hue_value: Option<String>,
    dark_theme_contrast_enabled: Option<String>,
    dark_theme_contrast_value: Option<String>,
    dark_theme_layered_chroma_enabled: Option<String>,
    dark_theme_layered_chroma_value: Option<String>,
    dark_theme_set: Option<String>,
    base_url: String,
    clean_urls: Option<String>,
    ftp_config_password: String,
    ftp_config_password_prompt: Option<String>,
    ftp_config_path: String,
    ftp_config_port: String,
    ftp_config_secure: Option<String>,
    ftp_config_server: String,
    ftp_config_user: String,
    language: String,
    light_theme_accent_brightening_enabled: Option<String>,
    light_theme_accent_brightening_value: Option<String>,
    light_theme_accent_chroma_enabled: Option<String>,
    light_theme_accent_chroma_value: Option<String>,
    light_theme_accent_hue_enabled: Option<String>,
    light_theme_accent_hue_value: Option<String>,
    light_theme_base_chroma_enabled: Option<String>,
    light_theme_base_chroma_value: Option<String>,
    light_theme_base_hue_enabled: Option<String>,
    light_theme_base_hue_value: Option<String>,
    light_theme_contrast_enabled: Option<String>,
    light_theme_contrast_value: Option<String>,
    light_theme_layered_chroma_enabled: Option<String>,
    light_theme_layered_chroma_value: Option<String>,
    light_theme_set: Option<String>,
    poster_aspect: String,
    roundness_enabled: Option<String>,
    roundness_value: Option<String>,
    ssh_config_password: String,
    ssh_config_password_prompt: Option<String>,
    ssh_config_path: String,
    ssh_config_port: String,
    ssh_config_private_key: String,
    ssh_config_public_key: String,
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
    let ftp_config_password_prompt = form.ftp_config_password_prompt.is_some();
    let ftp_config_path_trimmed = form.ftp_config_path.trim();
    let ftp_config_secure = form.ftp_config_secure.is_some();
    let ftp_config_server_trimmed = form.ftp_config_server.trim();
    let ftp_config_user_trimmed = form.ftp_config_user.trim();
    let language_trimmed = form.language.trim();
    let poster_aspect_trimmed = form.poster_aspect.trim();
    let ssh_config_password_trimmed = form.ssh_config_password.trim();
    let ssh_config_password_prompt = form.ssh_config_password_prompt.is_some();
    let ssh_config_path_trimmed = form.ssh_config_path.trim();
    let ssh_config_private_key_trimmed = form.ssh_config_private_key.trim();
    let ssh_config_public_key_trimmed = form.ssh_config_public_key.trim();
    let ssh_config_server_trimmed = form.ssh_config_server.trim();
    let ssh_config_user_trimmed = form.ssh_config_user.trim();
    let theme = match Theme::from_key(&form.theme) {
        Ok(theme) => theme,
        Err(_) => Theme::default()
    };

    fn unwrap_theme_setting<T>(
        enabled: &Option<String>,
        value: &Option<String>
    ) -> Option<T> where
        T: FromStr,
        <T as FromStr>::Err: Debug
    {
        enabled.as_ref().map(|_| value.as_ref().unwrap().parse::<T>().unwrap())
    }

    let dark_theme_accent_brightening = unwrap_theme_setting(&form.dark_theme_accent_brightening_enabled, &form.dark_theme_accent_brightening_value);
    let dark_theme_accent_chroma = unwrap_theme_setting(&form.dark_theme_accent_chroma_enabled, &form.dark_theme_accent_chroma_value);
    let dark_theme_accent_hue = unwrap_theme_setting(&form.dark_theme_accent_hue_enabled, &form.dark_theme_accent_hue_value);
    let dark_theme_base_chroma = unwrap_theme_setting(&form.dark_theme_base_chroma_enabled, &form.dark_theme_base_chroma_value);
    let dark_theme_base_hue = unwrap_theme_setting(&form.dark_theme_base_hue_enabled, &form.dark_theme_base_hue_value);
    let dark_theme_contrast = unwrap_theme_setting(&form.dark_theme_contrast_enabled, &form.dark_theme_contrast_value);
    let dark_theme_layered_chroma = unwrap_theme_setting(&form.dark_theme_layered_chroma_enabled, &form.dark_theme_layered_chroma_value);
    let dark_theme_set = form.dark_theme_set.is_some();

    let light_theme_accent_brightening = unwrap_theme_setting(&form.light_theme_accent_brightening_enabled, &form.light_theme_accent_brightening_value);
    let light_theme_accent_chroma = unwrap_theme_setting(&form.light_theme_accent_chroma_enabled, &form.light_theme_accent_chroma_value);
    let light_theme_accent_hue = unwrap_theme_setting(&form.light_theme_accent_hue_enabled, &form.light_theme_accent_hue_value);
    let light_theme_base_chroma = unwrap_theme_setting(&form.light_theme_base_chroma_enabled, &form.light_theme_base_chroma_value);
    let light_theme_base_hue = unwrap_theme_setting(&form.light_theme_base_hue_enabled, &form.light_theme_base_hue_value);
    let light_theme_contrast = unwrap_theme_setting(&form.light_theme_contrast_enabled, &form.light_theme_contrast_value);
    let light_theme_layered_chroma = unwrap_theme_setting(&form.light_theme_layered_chroma_enabled, &form.light_theme_layered_chroma_value);
    let light_theme_set = form.light_theme_set.is_some();

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

    let roundness = form.roundness_enabled
        .as_ref()
        .map(|_| {
            form.roundness_value
                .as_ref()
                .unwrap()
                .parse::<u8>().unwrap()
        });

    let ftp_config_port_trimmed = form.ftp_config_port.trim();
    let ftp_config_port = if ftp_config_port_trimmed.is_empty() {
        None
    } else {
        Some(
            ftp_config_port_trimmed.parse::<u16>().map_err(|err| err.to_string())
        )
    };

    let ssh_config_port_trimmed = form.ssh_config_port.trim();
    let ssh_config_port = if ssh_config_port_trimmed.is_empty() {
        None
    } else {
        Some(
            ssh_config_port_trimmed.parse::<u16>().map_err(|err| err.to_string())
        )
    };

    if base_url.as_ref().is_some_and(|base_url| base_url.is_err()) ||
        poster_aspect.is_err() ||
        ftp_config_port.as_ref().is_some_and(|ftp_config_port| ftp_config_port.is_err()) ||
        ssh_config_port.as_ref().is_some_and(|ssh_config_port| ssh_config_port.is_err())
    {
        let site = context.get_site();

        let base_url_err = base_url.and_then(|base_url| base_url.err().map(|err| err.to_string()));
        let ftp_config_port_err = ftp_config_port.and_then(|ftp_config_port| ftp_config_port.err());
        let ssh_config_port_err = ssh_config_port.and_then(|ssh_config_port| ssh_config_port.err());

        let form_feedback = UpdateFormFeedback {
            base_url: Field::validated(base_url_trimmed, base_url_err),
            clean_urls,
            dark_theme_accent_brightening,
            dark_theme_accent_chroma,
            dark_theme_accent_hue,
            dark_theme_base_chroma,
            dark_theme_base_hue,
            dark_theme_contrast,
            dark_theme_layered_chroma,
            dark_theme_set,
            ftp_config_password: Field::valid(ftp_config_password_trimmed),
            ftp_config_password_prompt,
            ftp_config_path: Field::valid(ftp_config_path_trimmed),
            ftp_config_port: Field::validated(ftp_config_port_trimmed, ftp_config_port_err),
            ftp_config_secure,
            ftp_config_server: Field::valid(ftp_config_server_trimmed),
            ftp_config_user: Field::valid(ftp_config_user_trimmed),
            language: Field::valid(language_trimmed),
            light_theme_accent_brightening,
            light_theme_accent_chroma,
            light_theme_accent_hue,
            light_theme_base_chroma,
            light_theme_base_hue,
            light_theme_contrast,
            light_theme_layered_chroma,
            light_theme_set,
            poster_aspect: Field::validated(poster_aspect_trimmed, poster_aspect.err()),
            roundness,
            ssh_config_password: Field::valid(ssh_config_password_trimmed),
            ssh_config_password_prompt,
            ssh_config_path: Field::valid(ssh_config_path_trimmed),
            ssh_config_port: Field::validated(ssh_config_port_trimmed, ssh_config_port_err),
            ssh_config_private_key: Field::valid(ssh_config_private_key_trimmed),
            ssh_config_public_key: Field::valid(ssh_config_public_key_trimmed),
            ssh_config_server: Field::valid(ssh_config_server_trimmed),
            ssh_config_user: Field::valid(ssh_config_user_trimmed),
            theme
        };

        let session_state = SessionState::all(&context.server_mode, &session);
        Either::Right(edit_page(&context, Some(form_feedback), &session_state, &site))
    } else {
        fn to_option<T: for<'a> From<&'a str>>(trimmed: &str) -> Option<T> {
            match trimmed.is_empty() {
                true => None,
                false => Some(trimmed.into())
            }
        }

        let language = if language_trimmed.is_empty() {
            Language::default()
        } else {
            Language::from_code(language_trimmed)
        };

        let mutation = |site_mut: &mut Site| {
            if dark_theme_set {
                let accent_brightening = dark_theme_accent_brightening.map(|accent_brightening| Percent::new(true, accent_brightening));
                let accent_chroma = dark_theme_accent_chroma.map(|accent_chroma| Percent::new(true, accent_chroma));
                let accent_hue = dark_theme_accent_hue.map(|accent_hue| Degrees::new(Some("deg"), accent_hue));
                let base_chroma = dark_theme_base_chroma.map(|base_chroma| Percent::new(true, base_chroma));
                let base_hue = dark_theme_base_hue.map(|base_hue| Degrees::new(Some("deg"), base_hue));
                let contrast = dark_theme_contrast.map(|contrast| Percent::new(true, contrast));
                let layered_chroma = dark_theme_layered_chroma.map(|layered_chroma| Percent::new(true, layered_chroma));

                let dark_theme = ThemePaint {
                    accent_brightening,
                    accent_chroma,
                    accent_hue,
                    base_chroma,
                    base_hue,
                    contrast,
                    layered_chroma
                };

                site_mut.dark_theme = Some(dark_theme);
            } else {
                site_mut.dark_theme = None;
            }

            if light_theme_set {
                let accent_brightening = light_theme_accent_brightening.map(|accent_brightening| Percent::new(true, accent_brightening));
                let accent_chroma = light_theme_accent_chroma.map(|accent_chroma| Percent::new(true, accent_chroma));
                let accent_hue = light_theme_accent_hue.map(|accent_hue| Degrees::new(Some("deg"), accent_hue));
                let base_chroma = light_theme_base_chroma.map(|base_chroma| Percent::new(true, base_chroma));
                let base_hue = light_theme_base_hue.map(|base_hue| Degrees::new(Some("deg"), base_hue));
                let contrast = light_theme_contrast.map(|contrast| Percent::new(true, contrast));
                let layered_chroma = light_theme_layered_chroma.map(|layered_chroma| Percent::new(true, layered_chroma));

                let light_theme = ThemePaint {
                    accent_brightening,
                    accent_chroma,
                    accent_hue,
                    base_chroma,
                    base_hue,
                    contrast,
                    layered_chroma
                };

                site_mut.light_theme = Some(light_theme);
            } else {
                site_mut.light_theme = None;
            }

            site_mut.base_url = base_url.map(|base_url| base_url.unwrap());
            site_mut.clean_urls = clean_urls;
            site_mut.ftp_config.password = to_option(ftp_config_password_trimmed);
            site_mut.ftp_config.password_prompt = Some(ftp_config_password_prompt);
            site_mut.ftp_config.path = to_option(ftp_config_path_trimmed);
            site_mut.ftp_config.port = ftp_config_port.and_then(|ftp_config_port| ftp_config_port.ok());
            site_mut.ftp_config.secure = Some(ftp_config_secure);
            site_mut.ftp_config.server = to_option(ftp_config_server_trimmed);
            site_mut.ftp_config.user = to_option(ftp_config_user_trimmed);
            site_mut.language = language;
            site_mut.poster_aspect = poster_aspect.unwrap();
            site_mut.roundness = roundness.map(|roundness| Percent::new(true, roundness));
            site_mut.ssh_config.password = to_option(ssh_config_password_trimmed);
            site_mut.ssh_config.password_prompt = Some(ssh_config_password_prompt);
            site_mut.ssh_config.path = to_option(ssh_config_path_trimmed);
            site_mut.ssh_config.port = ssh_config_port.and_then(|ssh_config_port| ssh_config_port.ok());
            site_mut.ssh_config.private_key = to_option(ssh_config_private_key_trimmed);
            site_mut.ssh_config.public_key = to_option(ssh_config_public_key_trimmed);
            site_mut.ssh_config.server = to_option(ssh_config_server_trimmed);
            site_mut.ssh_config.user = to_option(ssh_config_user_trimmed);

            // TODO: No way to unset (or not at all set) this right now, but
            // as it's going away anyway ...
            site_mut.theme = Some(theme);
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

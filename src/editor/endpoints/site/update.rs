// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use serde_derive::Deserialize;

use crate::{
    AspectRatio,
    Context,
    Field,
    Language,
    Site,
    SiteUrl,
    Theme
};
use crate::editor::routes;

use super::edit::edit_page;
use super::edit::UpdateFormFeedback;

#[derive(Deserialize)]
pub struct UpdateForm {
    base_url: String,
    clean_urls: Option<String>,
    deployment_path: String,
    deployment_server: String,
    deployment_user: String,
    language: String,
    poster_aspect: String,
    theme: String
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let base_url_trimmed = form.base_url.trim();
    let clean_urls = form.clean_urls.is_some();
    let deployment_path_trimmed = form.deployment_path.trim();
    let deployment_server_trimmed = form.deployment_server.trim();
    let deployment_user_trimmed = form.deployment_user.trim();
    let language_trimmed = form.language.trim();
    let poster_aspect_trimmed = form.poster_aspect.trim();
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

    if base_url.as_ref().is_some_and(|base_url| base_url.is_err()) || poster_aspect.is_err() {
        let site = context.get_site();

        let form_feedback = UpdateFormFeedback {
            base_url: Field::validated(base_url_trimmed, base_url.and_then(|base_url| base_url.err().map(|err| err.to_string()))),
            clean_urls,
            deployment_path: Field::valid(deployment_path_trimmed),
            deployment_server: Field::valid(deployment_server_trimmed),
            deployment_user: Field::valid(deployment_user_trimmed),
            language: Field::valid(language_trimmed),
            poster_aspect: Field::validated(poster_aspect_trimmed, poster_aspect.err()),
            theme
        };

        Either::Right(edit_page(&context, Some(form_feedback), &language, &site))
    } else {
        let deployment_path = if deployment_path_trimmed.is_empty() {
            None
        } else {
            Some(deployment_path_trimmed.to_string())
        };

        let deployment_server = if deployment_server_trimmed.is_empty() {
            None
        } else {
            Some(deployment_server_trimmed.to_string())
        };

        let deployment_user = if deployment_user_trimmed.is_empty() {
            None
        } else {
            Some(deployment_user_trimmed.to_string())
        };

        let language = if language_trimmed.is_empty() {
            Language::default()
        } else {
            Language::from_code(language_trimmed)
        };

        let mutation = |site_mut: &mut Site| {
            site_mut.base_url = base_url.map(|base_url| base_url.unwrap());
            site_mut.clean_urls = clean_urls;
            site_mut.deploy_config.path = deployment_path;
            site_mut.deploy_config.server = deployment_server;
            site_mut.deploy_config.user = deployment_user;
            site_mut.language = language;
            site_mut.poster_aspect = poster_aspect.unwrap();
            site_mut.theme = theme;
            site_mut.write_manifest(&context);
        };

        context.update_site(mutation);

        // Banner/Poster assets need recomputation when site.poster_aspect
        // changes (which could have happened here).
        context.compute_job_queue();

        Either::Left(Redirect::to(routes::SITE_EDIT).see_other())
    }
}

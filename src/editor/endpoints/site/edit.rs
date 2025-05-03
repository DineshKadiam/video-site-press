// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{
    AspectRatio,
    Context,
    Field,
    Language,
    Site,
    SitePath,
    Theme
};
use crate::editor::{Checkbox, Input, Location, Select};
use crate::editor::routes;
use crate::editor::widgets::layout;

pub struct UpdateFormFeedback {
    pub base_url: Field,
    pub clean_urls: bool,
    pub deployment_path: Field,
    pub deployment_server: Field,
    pub deployment_user: Field,
    pub language: Field,
    pub poster_aspect: Field,
    pub theme: Theme
}

pub async fn edit(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();

    let site = context.get_site();
    edit_page(&context, None, &language, &site)
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<UpdateFormFeedback>,
    language: &Language,
    site: &Site
) -> HttpResponse {
    let translations = &language.translations;

    let errors = if site.errors.is_empty() {
        String::new()
    } else {
        site.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let mut clean_urls_checkbox = Checkbox::new("clean_urls", translations.clean_urls_label);
    let mut input_base_url = Input::url("base_url", translations.public_address_label);
    let mut input_deployment_path = Input::text("deployment_path", translations.deployment_path_label);
    let mut input_deployment_server = Input::text("deployment_server", translations.deployment_server_label);
    let mut input_deployment_user = Input::text("deployment_user", translations.deployment_user_label);
    let mut input_language = Input::text("language", translations.language_label);
    let mut input_poster_aspect = Input::text("poster_aspect", translations.poster_aspect_label);

    let mut select_theme = Select::new("theme", translations.theme);
    for (key, label) in Theme::variants(translations) {
        select_theme.option(key, label);
    }

    // TODO: For better usability, include detailed hints (help texts, possibly toggeable, hidden by default)
    //       that give very clear indication about what base url, deployment path/server/user etc. are and
    //       what to pay attention to.
    let t_not_saved = translations.not_saved;
    let feedback = if let Some(unsaved) = &form_feedback {
        clean_urls_checkbox.checked(unsaved.clean_urls);
        input_base_url.validated_value(&unsaved.base_url);
        input_deployment_path.validated_value(&unsaved.deployment_path);
        input_deployment_server.validated_value(&unsaved.deployment_server);
        input_deployment_user.validated_value(&unsaved.deployment_user);
        input_language.validated_value(&unsaved.language);
        input_poster_aspect.validated_value(&unsaved.poster_aspect);
        select_theme.selected(unsaved.theme.key);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let language = if site.language.default { None } else { Some(&site.language.code) };
        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => Some(representation),
            AspectRatio::Default => None
        };

        clean_urls_checkbox.checked(site.clean_urls);
        input_base_url.value_option(&site.base_url.as_ref().map(|url| url.representation.clone()));
        input_deployment_path.value_option(&site.deploy_config.path);
        input_deployment_server.value_option(&site.deploy_config.server);
        input_deployment_user.value_option(&site.deploy_config.user);
        input_language.value_option(&language);
        input_poster_aspect.value_option(&poster_aspect);
        select_theme.selected(site.theme.key);

        String::new()
    };

    let t_deployment_is_currently_supported_text = translations.deployment_is_currently_supported_text;
    let t_general = translations.general;
    let t_site_settings = translations.site_settings;
    let t_deployment = translations.deployment;
    let t_update_site = translations.update_site;

    let update_action = routes::SITE_EDIT;

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_site_settings}</h2>

            {errors}

            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                <h3>{t_general}</h3>
                {input_language}
                {input_poster_aspect}
                {select_theme}
                {clean_urls_checkbox}

                <h3>{t_deployment}</h3>
                <p>{t_deployment_is_currently_supported_text}</p>
                {input_base_url}
                {input_deployment_path}
                {input_deployment_server}
                {input_deployment_user}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_site}</button> {feedback}
                </div>
            </form>
        </div>
    "#);

    let html = layout(
        &body,
        context,
        language,
        Location::site(&SitePath::root_path()),
        &site,
        t_site_settings
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

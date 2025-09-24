// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use hyper8_core::{
    AspectRatio,
    Site,
    SitePath,
    Theme
};

use crate::{
    Checkbox,
    Context,
    Field,
    Input,
    Location,
    Select,
    SessionState
};
use crate::routes;
use crate::widgets::layout;

pub struct UpdateFormFeedback {
    pub base_url: Field,
    pub clean_urls: bool,
    pub ftp_config_password: Field,
    pub ftp_config_path: Field,
    pub ftp_config_port: Field,
    pub ftp_config_secure: bool,
    pub ftp_config_server: Field,
    pub ftp_config_user: Field,
    pub language: Field,
    pub poster_aspect: Field,
    pub ssh_config_path: Field,
    pub ssh_config_server: Field,
    pub ssh_config_user: Field,
    pub theme: Theme
}

pub async fn edit(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    let site = context.get_site();
    edit_page(&context, None, &session_state, &site)
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<UpdateFormFeedback>,
    session_state: &SessionState,
    site: &Site
) -> HttpResponse {
    let translations = &session_state.language.translations;

    let manifest_errors = if site.manifest_errors.is_empty() {
        String::new()
    } else {
        site.manifest_errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let mut checkbox_clean_urls = Checkbox::new("clean_urls", translations.clean_urls_label);
    let mut checkbox_ftp_config_secure = Checkbox::new("ftp_config_secure", translations.ftp_config_secure_label);
    let mut input_base_url = Input::url("base_url", translations.public_address_label);
    let mut input_ftp_config_password = Input::text("ftp_config_password", translations.ftp_config_password_label);
    let mut input_ftp_config_path = Input::text("ftp_config_path", translations.ftp_config_path_label);
    let mut input_ftp_config_port = Input::number("ftp_config_port", translations.ftp_config_port_label);
    let mut input_ftp_config_server = Input::text("ftp_config_server", translations.ftp_config_server_label);
    let mut input_ftp_config_user = Input::text("ftp_config_user", translations.ftp_config_user_label);
    let mut input_language = Input::text("language", translations.language_label);
    let mut input_poster_aspect = Input::text("poster_aspect", translations.poster_aspect_label);
    let mut input_ssh_config_path = Input::text("ssh_config_path", translations.ssh_config_path_label);
    let mut input_ssh_config_server = Input::text("ssh_config_server", translations.ssh_config_server_label);
    let mut input_ssh_config_user = Input::text("ssh_config_user", translations.ssh_config_user_label);

    let mut select_theme = Select::new("theme", translations.theme);
    for (theme, label) in Theme::variants(translations) {
        select_theme.option(theme.key(), label);
    }

    // TODO: For better usability, include detailed hints (help texts, possibly toggeable, hidden by default)
    //       that give very clear indication about what base url, deployment path/server/user etc. are and
    //       what to pay attention to.
    let t_not_saved = translations.not_saved;
    let feedback = if let Some(unsaved) = &form_feedback {
        checkbox_clean_urls.checked(unsaved.clean_urls);
        checkbox_ftp_config_secure.checked(unsaved.ftp_config_secure);
        input_base_url.validated_value(&unsaved.base_url);
        input_ftp_config_password.validated_value(&unsaved.ftp_config_password);
        input_ftp_config_path.validated_value(&unsaved.ftp_config_path);
        input_ftp_config_port.validated_value(&unsaved.ftp_config_port);
        input_ftp_config_server.validated_value(&unsaved.ftp_config_server);
        input_ftp_config_user.validated_value(&unsaved.ftp_config_user);
        input_language.validated_value(&unsaved.language);
        input_poster_aspect.validated_value(&unsaved.poster_aspect);
        input_ssh_config_path.validated_value(&unsaved.ssh_config_path);
        input_ssh_config_server.validated_value(&unsaved.ssh_config_server);
        input_ssh_config_user.validated_value(&unsaved.ssh_config_user);
        select_theme.selected(unsaved.theme.key());

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let language = if site.language.default { None } else { Some(&site.language.code) };
        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => Some(representation),
            AspectRatio::Default => None
        };

        let ftp_config_port = site.ftp_config.port.as_ref().map(|number| number.to_string());

        checkbox_clean_urls.checked(site.clean_urls);
        checkbox_ftp_config_secure.checked(site.ftp_config.secure());
        input_base_url.value_option(&site.base_url.as_ref().map(|url| url.representation.clone()));
        input_ftp_config_password.value_option(&site.ftp_config.password);
        input_ftp_config_path.value_option(&site.ftp_config.path);
        input_ftp_config_port.value_option(&ftp_config_port);
        input_ftp_config_server.value_option(&site.ftp_config.server);
        input_ftp_config_user.value_option(&site.ftp_config.user);
        input_language.value_option(&language);
        input_poster_aspect.value_option(&poster_aspect);
        input_ssh_config_path.value_option(&site.ssh_config.path);
        input_ssh_config_server.value_option(&site.ssh_config.server);
        input_ssh_config_user.value_option(&site.ssh_config.user);
        select_theme.selected(site.theme.key());

        String::new()
    };

    let t_ftp_deployment = translations.ftp_deployment;
    let t_general = translations.general;
    let t_rsync_deployment = translations.rsync_deployment;
    let t_rsync_deployment_text = translations.rsync_deployment_text;
    let t_site_settings = translations.site_settings;
    let t_update_site = translations.update_site;

    let update_action = routes::SITE_EDIT;

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_site_settings}</h2>

            {manifest_errors}

            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                <h3>{t_general}</h3>
                {input_base_url}
                {input_language}
                {input_poster_aspect}
                {select_theme}
                {checkbox_clean_urls}

                <h3>{t_ftp_deployment}</h3>
                {input_ftp_config_server}
                {input_ftp_config_user}
                {input_ftp_config_password}
                {input_ftp_config_path}
                {input_ftp_config_port}
                {checkbox_ftp_config_secure}

                <h3>{t_rsync_deployment}</h3>
                <p>{t_rsync_deployment_text}</p>
                {input_ssh_config_server}
                {input_ssh_config_user}
                {input_ssh_config_path}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_site}</button> {feedback}
                </div>
            </form>
        </div>
    "#);

    let html = layout(
        &body,
        context,
        Location::site(&SitePath::root_path()),
        session_state,
        site,
        t_site_settings
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

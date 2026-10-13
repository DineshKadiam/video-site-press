// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use hyper8_core::{
    DEFAULT_ACCENT_BRIGHTENING,
    DEFAULT_ACCENT_CHROMA,
    DEFAULT_ACCENT_HUE,
    DEFAULT_BASE_CHROMA,
    DEFAULT_BASE_HUE,
    DEFAULT_CONTRAST,
    DEFAULT_LAYERED_CHROMA,
    DEFAULT_ROUNDNESS
};
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
    Layout,
    Location,
    Select,
    SessionState,
    Slider
};
use crate::routes;

pub struct UpdateFormFeedback {
    pub base_url: Field,
    pub clean_urls: bool,
    pub dark_theme_accent_brightening: Option<u8>,
    pub dark_theme_accent_chroma: Option<u8>,
    pub dark_theme_accent_hue: Option<u16>,
    pub dark_theme_base_chroma: Option<u8>,
    pub dark_theme_base_hue: Option<u16>,
    pub dark_theme_contrast: Option<u8>,
    pub dark_theme_layered_chroma: Option<u8>,
    pub dark_theme_set: bool,
    pub ftp_config_password: Field,
    pub ftp_config_password_prompt: bool,
    pub ftp_config_path: Field,
    pub ftp_config_port: Field,
    pub ftp_config_secure: bool,
    pub ftp_config_server: Field,
    pub ftp_config_user: Field,
    pub language: Field,
    pub light_theme_accent_brightening: Option<u8>,
    pub light_theme_accent_chroma: Option<u8>,
    pub light_theme_accent_hue: Option<u16>,
    pub light_theme_base_chroma: Option<u8>,
    pub light_theme_base_hue: Option<u16>,
    pub light_theme_contrast: Option<u8>,
    pub light_theme_layered_chroma: Option<u8>,
    pub light_theme_set: bool,
    pub poster_aspect: Field,
    pub roundness: Option<u8>,
    pub ssh_config_password: Field,
    pub ssh_config_password_prompt: bool,
    pub ssh_config_path: Field,
    pub ssh_config_port: Field,
    pub ssh_config_private_key: Field,
    pub ssh_config_public_key: Field,
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
    let mut checkbox_dark_theme_set = Checkbox::new("dark_theme_set", translations.customize);
    let mut checkbox_ftp_config_secure = Checkbox::new("ftp_config_secure", translations.ftp_config_secure_label);
    let mut checkbox_ftp_config_password_prompt = Checkbox::new("ftp_config_password_prompt", translations.ftp_config_password_prompt_label);
    let mut checkbox_light_theme_set = Checkbox::new("light_theme_set", translations.customize);
    let mut checkbox_ssh_config_password_prompt = Checkbox::new("ssh_config_password_prompt", translations.ssh_config_password_prompt_label);
    let mut input_base_url = Input::url("base_url", translations.public_address_label);
    let mut input_ftp_config_password = Input::password("ftp_config_password", translations.ftp_config_password_label);
    let mut input_ftp_config_path = Input::text("ftp_config_path", translations.ftp_config_path_label);
    let mut input_ftp_config_port = Input::number("ftp_config_port", translations.ftp_config_port_label);
    let mut input_ftp_config_server = Input::text("ftp_config_server", translations.ftp_config_server_label);
    let mut input_ftp_config_user = Input::text("ftp_config_user", translations.ftp_config_user_label);
    let mut input_language = Input::text("language", translations.language_label);
    let mut input_poster_aspect = Input::text("poster_aspect", translations.poster_aspect_label);
    let mut input_ssh_config_password = Input::text("ssh_config_password", translations.ssh_config_password_label);
    let mut input_ssh_config_path = Input::text("ssh_config_path", translations.ssh_config_path_label);
    let mut input_ssh_config_port = Input::text("ssh_config_port", translations.ssh_config_port_label);
    let mut input_ssh_config_private_key = Input::text("ssh_config_private_key", translations.ssh_config_private_key_label);
    let mut input_ssh_config_public_key = Input::text("ssh_config_public_key", translations.ssh_config_public_key_label);
    let mut input_ssh_config_server = Input::text("ssh_config_server", translations.ssh_config_server_label);
    let mut input_ssh_config_user = Input::text("ssh_config_user", translations.ssh_config_user_label);

    let mut slider_dark_theme_accent_brightening = Slider::new(
        DEFAULT_ACCENT_BRIGHTENING,
        "dark_theme_accent_brightening",
        translations.accent_brightening,
        100,
        0
    );

    let mut slider_dark_theme_accent_chroma = Slider::new(
        DEFAULT_ACCENT_CHROMA,
        "dark_theme_accent_chroma",
        translations.accent_chroma,
        100,
        0
    );

    let mut slider_dark_theme_accent_hue = Slider::new(
        DEFAULT_ACCENT_HUE,
        "dark_theme_accent_hue",
        translations.accent_hue,
        360,
        0
    );

    let mut slider_dark_theme_base_chroma = Slider::new(
        DEFAULT_BASE_CHROMA,
        "dark_theme_base_chroma",
        translations.base_chroma,
        100,
        0
    );

    let mut slider_dark_theme_base_hue = Slider::new(
        DEFAULT_BASE_HUE,
        "dark_theme_base_hue",
        translations.base_hue,
        360,
        0
    );

    let mut slider_dark_theme_contrast = Slider::new(
        DEFAULT_CONTRAST,
        "dark_theme_contrast",
        translations.contrast,
        100,
        0
    );

    let mut slider_dark_theme_layered_chroma = Slider::new(
        DEFAULT_LAYERED_CHROMA,
        "dark_theme_layered_chroma",
        translations.layered_chroma,
        100,
        0
    );

    let mut slider_light_theme_accent_brightening = Slider::new(
        DEFAULT_ACCENT_BRIGHTENING,
        "light_theme_accent_brightening",
        translations.accent_brightening,
        100,
        0
    );

    let mut slider_light_theme_accent_chroma = Slider::new(
        DEFAULT_ACCENT_CHROMA,
        "light_theme_accent_chroma",
        translations.accent_chroma,
        100,
        0
    );

    let mut slider_light_theme_accent_hue = Slider::new(
        DEFAULT_ACCENT_HUE,
        "light_theme_accent_hue",
        translations.accent_hue,
        360,
        0
    );

    let mut slider_light_theme_base_chroma = Slider::new(
        DEFAULT_BASE_CHROMA,
        "light_theme_base_chroma",
        translations.base_chroma,
        100,
        0
    );

    let mut slider_light_theme_base_hue = Slider::new(
        DEFAULT_BASE_HUE,
        "light_theme_base_hue",
        translations.base_hue,
        360,
        0
    );

    let mut slider_light_theme_contrast = Slider::new(
        DEFAULT_CONTRAST,
        "light_theme_contrast",
        translations.contrast,
        100,
        0
    );

    let mut slider_light_theme_layered_chroma = Slider::new(
        DEFAULT_LAYERED_CHROMA,
        "light_theme_layered_chroma",
        translations.layered_chroma,
        100,
        0
    );

    let mut slider_roundness = Slider::new(
        DEFAULT_ROUNDNESS,
        "roundness",
        translations.roundness,
        100,
        0
    );

    let mut select_theme = Select::new("theme", translations.theme);
    for (theme, label) in Theme::variants(translations) {
        select_theme.option(theme.key(), label);
    }

    let t_not_saved = translations.not_saved;
    let feedback = if let Some(unsaved) = &form_feedback {
        checkbox_clean_urls.checked(unsaved.clean_urls);
        checkbox_dark_theme_set.checked(unsaved.dark_theme_set);
        checkbox_light_theme_set.checked(unsaved.light_theme_set);
        checkbox_ftp_config_secure.checked(unsaved.ftp_config_secure);
        checkbox_ftp_config_password_prompt.checked(unsaved.ftp_config_password_prompt);
        checkbox_ssh_config_password_prompt.checked(unsaved.ssh_config_password_prompt);
        input_base_url.validated_value(&unsaved.base_url);
        input_ftp_config_password.validated_value(&unsaved.ftp_config_password);
        input_ftp_config_path.validated_value(&unsaved.ftp_config_path);
        input_ftp_config_port.validated_value(&unsaved.ftp_config_port);
        input_ftp_config_server.validated_value(&unsaved.ftp_config_server);
        input_ftp_config_user.validated_value(&unsaved.ftp_config_user);
        input_language.validated_value(&unsaved.language);
        input_poster_aspect.validated_value(&unsaved.poster_aspect);
        input_ssh_config_password.validated_value(&unsaved.ssh_config_password);
        input_ssh_config_path.validated_value(&unsaved.ssh_config_path);
        input_ssh_config_port.validated_value(&unsaved.ssh_config_port);
        input_ssh_config_private_key.validated_value(&unsaved.ssh_config_private_key);
        input_ssh_config_public_key.validated_value(&unsaved.ssh_config_public_key);
        input_ssh_config_server.validated_value(&unsaved.ssh_config_server);
        input_ssh_config_user.validated_value(&unsaved.ssh_config_user);
        slider_dark_theme_accent_brightening.value(unsaved.dark_theme_accent_brightening);
        slider_dark_theme_accent_chroma.value(unsaved.dark_theme_accent_chroma);
        slider_dark_theme_accent_hue.value(unsaved.dark_theme_accent_hue);
        slider_dark_theme_base_chroma.value(unsaved.dark_theme_base_chroma);
        slider_dark_theme_base_hue.value(unsaved.dark_theme_base_hue);
        slider_dark_theme_contrast.value(unsaved.dark_theme_contrast);
        slider_dark_theme_layered_chroma.value(unsaved.dark_theme_layered_chroma);
        slider_light_theme_accent_brightening.value(unsaved.light_theme_accent_brightening);
        slider_light_theme_accent_chroma.value(unsaved.light_theme_accent_chroma);
        slider_light_theme_accent_hue.value(unsaved.light_theme_accent_hue);
        slider_light_theme_base_chroma.value(unsaved.light_theme_base_chroma);
        slider_light_theme_base_hue.value(unsaved.light_theme_base_hue);
        slider_light_theme_contrast.value(unsaved.light_theme_contrast);
        slider_light_theme_layered_chroma.value(unsaved.light_theme_layered_chroma);
        slider_roundness.value(unsaved.roundness);
        select_theme.selected(unsaved.theme.key());

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let language = if site.language.default { None } else { Some(&site.language.code) };
        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => Some(representation),
            AspectRatio::Default => None
        };

        let dark_theme_accent_brightening = site.dark_theme.as_ref().map(|theme_paint| theme_paint.accent_brightening());
        let dark_theme_accent_chroma = site.dark_theme.as_ref().map(|theme_paint| theme_paint.accent_chroma());
        let dark_theme_accent_hue = site.dark_theme.as_ref().map(|theme_paint| theme_paint.accent_hue());
        let dark_theme_base_chroma = site.dark_theme.as_ref().map(|theme_paint| theme_paint.base_chroma());
        let dark_theme_base_hue = site.dark_theme.as_ref().map(|theme_paint| theme_paint.base_hue());
        let dark_theme_contrast = site.dark_theme.as_ref().map(|theme_paint| theme_paint.contrast());
        let dark_theme_layered_chroma = site.dark_theme.as_ref().map(|theme_paint| theme_paint.layered_chroma());

        let light_theme_accent_brightening = site.light_theme.as_ref().map(|theme_paint| theme_paint.accent_brightening());
        let light_theme_accent_chroma = site.light_theme.as_ref().map(|theme_paint| theme_paint.accent_chroma());
        let light_theme_accent_hue = site.light_theme.as_ref().map(|theme_paint| theme_paint.accent_hue());
        let light_theme_base_chroma = site.light_theme.as_ref().map(|theme_paint| theme_paint.base_chroma());
        let light_theme_base_hue = site.light_theme.as_ref().map(|theme_paint| theme_paint.base_hue());
        let light_theme_contrast = site.light_theme.as_ref().map(|theme_paint| theme_paint.contrast());
        let light_theme_layered_chroma = site.light_theme.as_ref().map(|theme_paint| theme_paint.layered_chroma());

        let roundness = site.roundness.map(|roundness| roundness.value());

        let ftp_config_port = site.ftp_config.port.as_ref().map(|number| number.to_string());
        let ssh_config_port = site.ssh_config.port.as_ref().map(|number| number.to_string());

        checkbox_clean_urls.checked(site.clean_urls);
        checkbox_dark_theme_set.checked(site.dark_theme.is_some());
        checkbox_light_theme_set.checked(site.light_theme.is_some());
        checkbox_ftp_config_secure.checked(site.ftp_config.secure());
        checkbox_ftp_config_password_prompt.checked(site.ftp_config.password_prompt());
        checkbox_ssh_config_password_prompt.checked(site.ssh_config.password_prompt());
        input_base_url.value_option(&site.base_url.as_ref().map(|url| url.representation.clone()));
        input_ftp_config_password.value_option(&site.ftp_config.password);
        input_ftp_config_path.value_option(&site.ftp_config.path);
        input_ftp_config_port.value_option(&ftp_config_port);
        input_ftp_config_server.value_option(&site.ftp_config.server);
        input_ftp_config_user.value_option(&site.ftp_config.user);
        input_language.value_option(&language);
        input_poster_aspect.value_option(&poster_aspect);
        input_ssh_config_password.value_option(&site.ssh_config.password);
        input_ssh_config_path.value_option(&site.ssh_config.path);
        input_ssh_config_port.value_option(&ssh_config_port);
        input_ssh_config_private_key.value_option(&site.ssh_config.private_key.as_ref().map(|path| path.display().to_string()));
        input_ssh_config_public_key.value_option(&site.ssh_config.public_key.as_ref().map(|path| path.display().to_string()));
        input_ssh_config_server.value_option(&site.ssh_config.server);
        input_ssh_config_user.value_option(&site.ssh_config.user);
        slider_dark_theme_accent_brightening.value(dark_theme_accent_brightening);
        slider_dark_theme_accent_chroma.value(dark_theme_accent_chroma);
        slider_dark_theme_accent_hue.value(dark_theme_accent_hue);
        slider_dark_theme_base_chroma.value(dark_theme_base_chroma);
        slider_dark_theme_base_hue.value(dark_theme_base_hue);
        slider_dark_theme_contrast.value(dark_theme_contrast);
        slider_dark_theme_layered_chroma.value(dark_theme_layered_chroma);
        slider_light_theme_accent_brightening.value(light_theme_accent_brightening);
        slider_light_theme_accent_chroma.value(light_theme_accent_chroma);
        slider_light_theme_accent_hue.value(light_theme_accent_hue);
        slider_light_theme_base_chroma.value(light_theme_base_chroma);
        slider_light_theme_base_hue.value(light_theme_base_hue);
        slider_light_theme_contrast.value(light_theme_contrast);
        slider_light_theme_layered_chroma.value(light_theme_layered_chroma);
        slider_roundness.value(roundness);
        select_theme.selected(site.theme().key());

        String::new()
    };

    let t_dark_color_scheme = translations.dark_color_scheme;
    let t_design = translations.design;
    let t_design_overlay = translations.design_overlay;
    let t_design_overlay_hint_text = translations.design_overlay_hint_text;
    let t_ftp_sync_settings = translations.ftp_sync_settings;
    let t_general = translations.general;
    let t_light_color_scheme = translations.light_color_scheme;
    let t_preview_with_design_overlay = translations.preview_with_design_overlay;
    let t_site_settings = translations.site_settings;
    let t_ssh_sync_settings = translations.ssh_sync_settings;
    let t_ssh_sync_settings_text = translations.ssh_sync_settings_text;
    let t_update_site = translations.update_site;

    let preview_with_design_overlay_action = routes::PREVIEW_WITH_DESIGN_OVERLAY;
    let update_action = routes::SITE_EDIT;

    let dark_theme_active = if checkbox_dark_theme_set.is_checked() { "active" } else { "" };
    let light_theme_active = if checkbox_light_theme_set.is_checked() { "active" } else { "" };

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_site_settings}</h2>

            {manifest_errors}

            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                <h3>{t_general}</h3>
                {input_base_url}
                {input_language}
                {checkbox_clean_urls}

                <h3>{t_design}</h3>
                {select_theme}
                {input_poster_aspect}

                <!-- TODO: Unhide experimental theme features eventually BEGIN -->
                <div aria-hidden="true" style="height: 0; overflow: hidden; width: 0; visibility: hidden;">

                {slider_roundness}

                <h4>{t_dark_color_scheme}</h4>
                {checkbox_dark_theme_set}
                <div class="theme_settings {dark_theme_active}" data-dark-theme>
                    {slider_dark_theme_accent_brightening}
                    {slider_dark_theme_accent_chroma}
                    {slider_dark_theme_accent_hue}
                    {slider_dark_theme_base_chroma}
                    {slider_dark_theme_base_hue}
                    {slider_dark_theme_contrast}
                    {slider_dark_theme_layered_chroma}
                </div>

                <h4>{t_light_color_scheme}</h4>
                {checkbox_light_theme_set}
                <div class="theme_settings {light_theme_active}" data-light-theme>
                    {slider_light_theme_accent_brightening}
                    {slider_light_theme_accent_chroma}
                    {slider_light_theme_accent_hue}
                    {slider_light_theme_base_chroma}
                    {slider_light_theme_base_hue}
                    {slider_light_theme_contrast}
                    {slider_light_theme_layered_chroma}
                </div>

                <h4>{t_design_overlay}</h4>
                <p>{t_design_overlay_hint_text}</p>
                <p>
                    <a href="{preview_with_design_overlay_action}" style="text-decoration: underline;" target="_blank">
                        {t_preview_with_design_overlay}
                    </a>
                </p>

                </div>
                <!-- TODO: Unhide experimental theme features eventually END -->

                <h3>{t_ftp_sync_settings}</h3>
                {input_ftp_config_server}
                {input_ftp_config_port}
                {input_ftp_config_path}
                {input_ftp_config_user}
                {input_ftp_config_password}
                {checkbox_ftp_config_password_prompt}
                {checkbox_ftp_config_secure}

                <h3>{t_ssh_sync_settings}</h3>
                <p>{t_ssh_sync_settings_text}</p>
                {input_ssh_config_server}
                {input_ssh_config_port}
                {input_ssh_config_path}
                {input_ssh_config_user}
                {input_ssh_config_password}
                {checkbox_ssh_config_password_prompt}
                {input_ssh_config_private_key}
                {input_ssh_config_public_key}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_site}</button> {feedback}
                </div>
            </form>
        </div>
    "#);

    let layout = Layout::new();

    let html = layout.render(
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

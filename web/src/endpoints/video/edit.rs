// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{
    PlatformIntegration,
    SitePath,
    Video
};

use crate::{
    Checkbox,
    Context,
    Field,
    Input,
    Location,
    Select,
    SessionState,
    Textarea
};
use crate::routes;
use crate::widgets::{
    layout,
    not_found,
    resource_actions
};

pub struct UpdateFormFeedback<'a> {
    pub copy_link: Option<bool>,
    pub description: &'a str,
    pub download: Option<bool>,
    pub embedding: Option<bool>,
    pub offline: bool,
    pub platform_integration: Option<PlatformIntegration>,
    pub release_date: Field,
    pub sort_number: Field,
    pub synopsis: Field,
    pub title: Field,
    pub unlisted: bool
}

pub async fn edit(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_video(&site_path) {
        Some(video) => edit_page(&context, None, &session_state, &video),
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<UpdateFormFeedback>,
    session_state: &SessionState,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;
    let t_not_saved = translations.not_saved;

    let mut errors = String::new();

    for dir_error in &video.dir_errors {
        errors.push_str(&format!(r#"<pre class="error">{dir_error}</pre>"#));
    }

    for manifest_error in &video.manifest_errors {
        errors.push_str(&format!(r#"<pre class="error">{manifest_error}</pre>"#));
    }

    let mut input_offline = Checkbox::new("offline", translations.offline);
    let mut input_release_date = Input::date("release_date", translations.release_date);
    let mut input_sort_number = Input::number("sort_number", translations.sort_number);
    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut select_copy_link = Select::new("copy_link", translations.copy_link);
    let mut select_download = Select::new("download", translations.downloads);
    let mut select_embedding = Select::new("embedding", translations.embedding);
    let mut select_platform_integration = Select::new("platform_integration", translations.automatic_link_previews_and_content_embedding_on_platforms);
    let mut textarea_description = Textarea::markdown_subset(translations, "description", translations.description);
    let mut textarea_synopsis = Textarea::synopsis(translations, "synopsis", translations.synopsis);

    let t_disabled = translations.disabled;
    let t_enabled = translations.enabled;

    let t_default_or_inherited = match video.site_path.is_root_path() {
        true => translations.default,
        false => translations.inherited
    };

    let t_copy_link_disabled_enabled = if video.copy_link_inherited() { t_enabled } else { t_disabled };
    select_copy_link.option("", format!("{t_copy_link_disabled_enabled} ({t_default_or_inherited})"));
    select_copy_link.option("enabled", t_enabled);
    select_copy_link.option("disabled", t_disabled);

    let t_download_disabled_enabled = if video.download_inherited() { t_enabled } else { t_disabled };
    select_download.option("", format!("{t_download_disabled_enabled} ({t_default_or_inherited})"));
    select_download.option("enabled", t_enabled);
    select_download.option("disabled", t_disabled);

    let t_embedding_disabled_enabled = if video.embedding_inherited() { t_enabled } else { t_disabled };
    select_embedding.option("", format!("{t_embedding_disabled_enabled} ({t_default_or_inherited})"));
    select_embedding.option("enabled", t_enabled);
    select_embedding.option("disabled", t_disabled);

    let platform_integration_inherited = video.platform_integration_inherited();
    let t_platform_integration_inherited_label = platform_integration_inherited.label(translations);
    select_platform_integration.option("", format!("{t_platform_integration_inherited_label} ({t_default_or_inherited})"));
    select_platform_integration.option("disabled", t_disabled);
    select_platform_integration.option("link_previews", translations.link_previews_only);
    select_platform_integration.option("content_embeds", translations.link_previews_and_content_embeds);

    textarea_description.id("video_description");

    if site.base_url.is_none() {
        let failure_icon = hyper8_icons::failure(translations.warning);
        let t_public_address_not_configured_text = translations.public_address_not_configured_text;
        let hint = format!("{failure_icon} {t_public_address_not_configured_text}");

        select_embedding.hint(&hint);
        select_platform_integration.hint(hint);
    }

    let feedback_update = if let Some(UpdateFormFeedback {
        copy_link,
        description,
        download,
        embedding,
        offline,
        platform_integration,
        release_date,
        sort_number,
        synopsis,
        title,
        unlisted
    }) = &form_feedback {
        let copy_link_value = match copy_link {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download_value = match download {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding_value = match embedding {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let platform_integration_value = match platform_integration {
            Some(platform_integration) => platform_integration.key(),
            None => ""
        };

        input_offline.checked(*offline);
        input_release_date.validated_value(release_date);
        input_sort_number.validated_value(sort_number);
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        select_copy_link.selected(copy_link_value);
        select_download.selected(download_value);
        select_embedding.selected(embedding_value);
        select_platform_integration.selected(platform_integration_value);
        textarea_description.value(description);
        textarea_synopsis.validated_value(synopsis);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let copy_link = match video.copy_link_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download = match video.download_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding = match video.embedding_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let platform_integration = match video.platform_integration_local() {
            Some(platform_integration) => platform_integration.key(),
            None => ""
        };

        let release_date = video.release_date.as_ref().map(|date| date.to_string());
        let sort_number = video.sort_number.as_ref().map(|number| number.to_string());

        input_offline.checked(video.offline_local());
        input_release_date.value_option(&release_date);
        input_sort_number.value_option(&sort_number);
        input_title.value_option(&video.title);
        input_unlisted.checked(video.unlisted_local());
        select_copy_link.selected(copy_link);
        select_download.selected(download);
        select_embedding.selected(embedding);
        select_platform_integration.selected(platform_integration);
        textarea_description.value_option(&video.description);
        textarea_synopsis.value_option(&video.synopsis);

        String::new()
    };

    let input_unlisted_conditionally_hidden = if !video.site_path.is_root_path() {
        input_unlisted.to_string()
    } else if video.unlisted_local() {
        String::from(r#"<input name="unlisted" type="hidden" value="checked">"#)
    } else {
        String::new()
    };

    let r_resource_actions = resource_actions(
        context,
        video.offline(),
        &site,
        &video.site_path,
        translations
    );

    let video_icon = hyper8_icons::video(translations.video);
    let video_title = video.title_or_slug_or_generic_label(translations);

    let t_update_video = translations.update_video;

    let update_action = routes::video_edit(video);

    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                {video_icon}
                <span>{video_title}</span>
            </h2>
            {r_resource_actions}
        </div>

        <div class="form_group">
            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                <!-- TODO: Expose/unhide synopsis if and when it makes sense again -->
                <div style="height: 0; opacity: 0; width: 0;">
                    {textarea_synopsis}
                </div>
                {textarea_description}
                <div class="form_split divide_3_1">
                    {input_release_date}
                    {input_sort_number}
                </div>
                {select_copy_link}
                {select_download}
                {select_embedding}
                {select_platform_integration}
                {input_offline}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_video}</button> {feedback_update}
                </div>
            </form>
        </div>
    "#);

    let t_video_settings = translations.video_settings;
    let video_title = video.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_video_settings} – {video_title}");
    let html = layout(
        &body,
        context,
        Location::edit(&video.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

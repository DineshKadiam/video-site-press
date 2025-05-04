// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use crate::{
    Context,
    Field,
    Language,
    PlatformIntegration,
    Playlist,
    SitePath
};
use crate::editor::{Checkbox, Input, Location, Select, Textarea};
use crate::editor::routes;
use crate::editor::widgets::{
    add_link_form,
    layout,
    links,
    not_found,
    resource_actions,
    upload_widget
};
use crate::icons;

pub enum PlaylistFormFeedback<'a> {
    CreateVideo {
        slug: Field,
        title: Field
    },
    Update {
        copy_link: Option<bool>,
        description: &'a str,
        download: Option<bool>,
        embedding: Option<bool>,
        feeds: Option<bool>,
        offline: bool,
        order: &'a str,
        platform_integration: Option<PlatformIntegration>,
        title: Field,
        unlisted: bool
    }
}

pub async fn edit(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_playlist(&site_path) {
        Some(playlist) => edit_page(&context, None, &language, &playlist),
        None => not_found(&context, &language, &site_path)
    }
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<PlaylistFormFeedback>,
    language: &Language,
    playlist: &Playlist
) -> HttpResponse {
    let site = context.get_site();
    let translations = &language.translations;

    let t_not_saved = translations.not_saved;

    let errors = if playlist.errors.is_empty() {
        String::new()
    } else {
        playlist.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let t_upload_banner = translations.upload_banner;
    let banner_upload_button = upload_widget(&routes::playlist_upload_banner(playlist), t_upload_banner);
    let banner_form = match &playlist.banner {
        Some(banner) => {
            let alt = if let Some(description) = &banner.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            // TODO: Could/should also use computed assets (also elsewhere, videos etc.)
            let file_name = &banner.file_name;
            let src = routes::poster(file_name, &playlist.site_path);

            let mut description = Textarea::image_description("description", translations.banner_description);

            description
                .id("banner_description")
                .value_option(&banner.description);

            let t_delete_banner = translations.delete_banner;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;

            let delete_banner_action = routes::playlist_delete_banner(playlist);
            let update_banner_action = routes::playlist_update_banner(playlist);
            format!(r#"
                <img {alt} class="thumbnail" src="{src}">
                <div class="button_group">
                    {banner_upload_button}
                    <form action="{delete_banner_action}" data-confirm data-confirm-label="{t_delete_banner}" data-confirm-text="{t_really_delete}" method="post">
                        <button class="danger">{t_delete_banner}</button>
                    </form>
                </div>
                <form action="{update_banner_action}" data-warn-discard method="post">
                    {description}
                    <button class="action">{t_update_description}</button>
                </form>
            "#)
        }
        None => banner_upload_button
    };

    let mut input_offline = Checkbox::new("offline", translations.offline);
    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut select_copy_link = Select::new("copy_link", translations.copy_link);
    let mut select_download = Select::new("download", translations.downloads);
    let mut select_embedding = Select::new("embedding", translations.embedding);
    let mut select_feeds = Select::new("feeds", translations.feeds);
    let mut select_order = Select::new("order", translations.order);
    let mut select_platform_integration = Select::new("platform_integration", translations.automatic_link_previews_and_content_embedding_on_platforms);
    let mut textarea_description = Textarea::markdown_subset(language, "description", translations.description);

    let t_disabled = translations.disabled;
    let t_enabled = translations.enabled;

    let t_default_or_inherited = match playlist.site_path.is_root_path() {
        true => translations.default,
        false => translations.inherited
    };

    let t_copy_link_disabled_enabled = if playlist.copy_link_inherited() { t_enabled } else { t_disabled };
    select_copy_link.option("", format!("{t_copy_link_disabled_enabled} ({t_default_or_inherited})"));
    select_copy_link.option("enabled", t_enabled);
    select_copy_link.option("disabled", t_disabled);

    let t_download_disabled_enabled = if playlist.download_inherited() { t_enabled } else { t_disabled };
    select_download.option("", format!("{t_download_disabled_enabled} ({t_default_or_inherited})"));
    select_download.option("enabled", t_enabled);
    select_download.option("disabled", t_disabled);

    let t_embedding_disabled_enabled = if playlist.embedding_inherited() { t_enabled } else { t_disabled };
    select_embedding.option("", format!("{t_embedding_disabled_enabled} ({t_default_or_inherited})"));
    select_embedding.option("enabled", t_enabled);
    select_embedding.option("disabled", t_disabled);

    let t_feeds_disabled_enabled = if playlist.feeds_inherited() { t_enabled } else { t_disabled };
    select_feeds.option("", format!("{t_feeds_disabled_enabled} ({t_default_or_inherited})"));
    select_feeds.option("enabled", t_enabled);
    select_feeds.option("disabled", t_disabled);

    select_order.option("default", translations.default_sort_number_or_release_date_ascending);
    select_order.option("release_date_asc", translations.release_date_ascending);
    select_order.option("release_date_desc", translations.release_date_descending);
    select_order.option("sort_number_asc", translations.sort_number_ascending);
    select_order.option("sort_number_desc", translations.sort_number_descending);
    select_order.option("title_asc", translations.title_ascending);
    select_order.option("title_desc", translations.title_descending);

    let platform_integration_inherited = playlist.platform_integration_inherited();
    let t_platform_integration_inherited_label = platform_integration_inherited.label(translations);
    select_platform_integration.option("", format!("{t_platform_integration_inherited_label} ({t_default_or_inherited})"));
    select_platform_integration.option("disabled", t_disabled);
    select_platform_integration.option("link_previews", translations.link_previews_only);
    select_platform_integration.option("content_embeds", translations.link_previews_and_content_embeds);

    textarea_description.id("playlist_description");

    if site.base_url.is_none() {
        let failure_icon = icons::failure(translations.warning);
        let t_public_address_not_configured_text = translations.public_address_not_configured_text;
        let hint = format!("{failure_icon} {t_public_address_not_configured_text}");

        select_embedding.hint(&hint);
        select_feeds.hint(&hint);
        select_platform_integration.hint(hint);
    }

    let feedback_update = if let Some(PlaylistFormFeedback::Update {
        copy_link,
        description,
        download,
        embedding,
        feeds,
        offline,
        order,
        platform_integration,
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

        let feeds_value = match feeds {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let platform_integration_value = match platform_integration {
            Some(platform_integration) => platform_integration.key(),
            None => ""
        };

        input_offline.checked(*offline);
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        select_copy_link.selected(copy_link_value);
        select_download.selected(download_value);
        select_embedding.selected(embedding_value);
        select_feeds.selected(feeds_value);
        select_order.selected(order);
        select_platform_integration.selected(platform_integration_value);
        textarea_description.value(description);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let copy_link = match playlist.copy_link_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download = match playlist.download_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding = match playlist.embedding_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let feeds = match playlist.feeds_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let platform_integration = match playlist.platform_integration_local() {
            Some(platform_integration) => platform_integration.key(),
            None => ""
        };

        input_offline.checked(playlist.offline_local());
        input_title.value_option(&playlist.title);
        input_unlisted.checked(playlist.unlisted_local());
        select_copy_link.selected(copy_link);
        select_download.selected(download);
        select_embedding.selected(embedding);
        select_feeds.selected(feeds);
        select_order.selected(playlist.order.key());
        select_platform_integration.selected(platform_integration);
        textarea_description.value_option(&playlist.description);

        String::new()
    };

    let t_upload_new_video = translations.upload_new_video;
    let upload_url = routes::upload_video(&playlist.site_path);
    let video_upload_button = upload_widget(&upload_url, t_upload_new_video);

    let input_unlisted_conditionally_hidden = if !playlist.site_path.is_root_path() {
        input_unlisted.to_string()
    } else if playlist.unlisted_local() {
        String::from(r#"<input name="unlisted" type="hidden" value="checked">"#)
    } else {
        String::new()
    };

    let new_video_form = {
        let mut input_slug_create_video = Input::permalink(
            "slug",
            &playlist.site_path,
            &site,
            translations
        );
        let mut input_title_create_video = Input::text("title", translations.title);

        // TODO: Needs custom client-side validation - either title or slug must be given.
        input_slug_create_video.id("slug_video");
        input_title_create_video.id("title_video");

        let feedback = if let Some(PlaylistFormFeedback::CreateVideo { slug, title }) = &form_feedback {
            input_slug_create_video.validated_value(slug);
            input_title_create_video.validated_value(title);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            String::new()
        };

        let t_create_video = translations.create_video;
        let t_new_video = translations.new_video;
        let t_you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided = &translations.you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided;

        let create_video_action = routes::playlist_create_video(playlist);
        formatdoc!(r#"
            <div class="form_group">
                <h2>{t_new_video}</h2>

                <form action="{create_video_action}" autocomplete="off" data-warn-discard method="post">
                    <p>
                        {t_you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided}
                    </p>
                    {input_title_create_video}
                    {input_slug_create_video}
                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button class="action">{t_create_video}</button> {feedback}
                    </div>
                </form>
            </div>
        "#)
    };

    let create_link_form = add_link_form(&routes::playlist_add_link(playlist), translations);
    let links_form = links(&playlist.links, &routes::playlist_delete_link(playlist), translations);

    let r_resource_actions = resource_actions(
        context,
        playlist.offline(),
        &site,
        &playlist.site_path,
        translations
    );

    let playlist_icon = icons::playlist(translations.playlist);
    let playlist_title = playlist.title_or_slug_or_generic_label(translations);

    let t_banner_image = translations.banner_image;

    let t_new_video = translations.new_video;
    let t_update_playlist = translations.update_playlist;

    let update_action = routes::playlist_edit(playlist);

    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                {playlist_icon}
                <span>{playlist_title}</span>
            </h2>
            {r_resource_actions}
        </div>

        <div class="form_group">
            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                {select_order}
                {select_copy_link}
                {select_download}
                {select_embedding}
                {select_feeds}
                {select_platform_integration}
                {input_offline}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_playlist}</button> {feedback_update}
                </div>
            </form>
        </div>

        {links_form}

        {create_link_form}

        {new_video_form}

        <div class="form_group">
            <h2>{t_new_video}</h2>

            {video_upload_button}
        </div>

        <div class="form_group">
            <h2>{t_banner_image}</h2>
            <p style="background: var(--warning-bg); border-radius: .5rem; color: var(--warning-fg); padding: 1rem;">
                Heads up: Banner images are not yet used in the layout,<br>but this will be included in a future version of Hyper 8
            </p>
            {banner_form}
        </div>
    "#);

    let html = layout(
        &body,
        context,
        language,
        Location::edit(&playlist.site_path),
        &site,
        playlist.title_or_slug_or_generic_label(translations)
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

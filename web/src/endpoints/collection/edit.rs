// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{
    Collection,
    PlatformIntegration,
    SitePath
};

use crate::actix::html_response;
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
    resource_actions,
    upload_widget
};

pub enum CollectionFormFeedback<'a> {
    CreatePlaylist {
        slug: Field,
        title: Field
    },
    CreateSubcollection {
        slug: Field,
        title: Field
    },
    CreateVideo {
        slug: Field,
        title: Field
    },
    None,
    Update {
        container_order: &'a str,
        copy_link: Option<bool>,
        description: &'a str,
        download: Option<bool>,
        embedding: Option<bool>,
        feeds: Option<bool>,
        offline: bool,
        platform_integration: Option<PlatformIntegration>,
        sort_number: Field,
        title: Field,
        unlisted: bool,
        video_order: &'a str
    }
}

pub async fn edit(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_collection(&site_path) {
        Some(collection) => {
            edit_page(
                &collection,
                &context,
                CollectionFormFeedback::None,
                &session_state
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn edit_page(
    collection: &Collection,
    context: &Context,
    form_feedback: CollectionFormFeedback,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;
    let t_not_saved = translations.not_saved;

    let mut errors = String::new();

    for dir_error in &collection.dir_errors {
        errors.push_str(&format!(r#"<pre class="error">{dir_error}</pre>"#));
    }

    for manifest_error in &collection.manifest_errors {
        errors.push_str(&format!(r#"<pre class="error">{manifest_error}</pre>"#));
    }

    let t_container_order = translations.container_order;
    let t_order_in_which_playlists_and_subcollections_appear = translations.order_in_which_playlists_and_subcollections_appear;
    let select_container_order_label = format!("{t_container_order} – {t_order_in_which_playlists_and_subcollections_appear}");

    let t_video_order = translations.video_order;
    let t_videos_in_a_collection_always_appear_at_the_top = translations.videos_in_a_collection_always_appear_at_the_top;
    let select_video_order_label = format!("{t_video_order} – {t_videos_in_a_collection_always_appear_at_the_top}");

    let mut input_offline = Checkbox::new("offline", translations.offline);
    let mut input_sort_number = Input::number("sort_number", translations.sort_number);
    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut select_container_order = Select::new("container_order", &select_container_order_label);
    let mut select_copy_link = Select::new("copy_link", translations.copy_link);
    let mut select_download = Select::new("download", translations.downloads);
    let mut select_embedding = Select::new("embedding", translations.embedding);
    let mut select_feeds = Select::new("feeds", translations.feeds);
    let mut select_platform_integration = Select::new("platform_integration", translations.automatic_link_previews_and_content_embedding_on_platforms);
    let mut select_video_order = Select::new("video_order", &select_video_order_label);
    let mut textarea_description = Textarea::markdown_subset(translations, "description", translations.description);

    let t_disabled = translations.disabled;
    let t_enabled = translations.enabled;

    let t_default_or_inherited = match collection.site_path.is_root_path() {
        true => translations.default,
        false => translations.inherited
    };

    select_container_order.option("default", translations.default_sort_number_ascending_or_latest_release_date_descending);
    select_container_order.option("sort_number_asc", translations.sort_number_ascending);
    select_container_order.option("sort_number_desc", translations.sort_number_descending);
    select_container_order.option("title_asc", translations.title_ascending);
    select_container_order.option("title_desc", translations.title_descending);

    let t_copy_link_inherited_enabled_disabled = if collection.copy_link_inherited() { t_enabled } else { t_disabled };
    select_copy_link.option("", format!("{t_copy_link_inherited_enabled_disabled} ({t_default_or_inherited})"));
    select_copy_link.option("enabled", t_enabled);
    select_copy_link.option("disabled", t_disabled);

    let t_download_inherited_enabled_disabled = if collection.download_inherited() { t_enabled } else { t_disabled };
    select_download.option("", format!("{t_download_inherited_enabled_disabled} ({t_default_or_inherited})"));
    select_download.option("enabled", t_enabled);
    select_download.option("disabled", t_disabled);

    let t_embedding_inherited_enabled_disabled = if collection.embedding_inherited() { t_enabled } else { t_disabled };
    select_embedding.option("", format!("{t_embedding_inherited_enabled_disabled} ({t_default_or_inherited})"));
    select_embedding.option("enabled", t_enabled);
    select_embedding.option("disabled", t_disabled);

    let t_feeds_inherited_enabled_disabled = if collection.feeds_inherited() { t_enabled } else { t_disabled };
    select_feeds.option("", format!("{t_feeds_inherited_enabled_disabled} ({t_default_or_inherited})"));
    select_feeds.option("enabled", t_enabled);
    select_feeds.option("disabled", t_disabled);

    let platform_integration_inherited = collection.platform_integration_inherited();
    let t_platform_integration_inherited_label = platform_integration_inherited.label(translations);
    select_platform_integration.option("", format!("{t_platform_integration_inherited_label} ({t_default_or_inherited})"));
    select_platform_integration.option("disabled", t_disabled);
    select_platform_integration.option("link_previews", translations.link_previews_only);
    select_platform_integration.option("content_embeds", translations.link_previews_and_content_embeds);

    // TODO: Same construction as in endpoints/playlist.rs, possibly encapsulate as shared helper
    select_video_order.option("default", translations.default_sort_number_or_release_date_ascending);
    select_video_order.option("release_date_asc", translations.release_date_ascending);
    select_video_order.option("release_date_desc", translations.release_date_descending);
    select_video_order.option("sort_number_asc", translations.sort_number_ascending);
    select_video_order.option("sort_number_desc", translations.sort_number_descending);
    select_video_order.option("title_asc", translations.title_ascending);
    select_video_order.option("title_desc", translations.title_descending);

    textarea_description.id("collection_description");

    if site.base_url.is_none() {
        let failure_icon = hyper8_icons::failure(translations.warning);
        let t_public_address_not_configured_text = translations.public_address_not_configured_text;
        let hint = format!("{failure_icon} {t_public_address_not_configured_text}");

        select_embedding.hint(&hint);
        select_feeds.hint(&hint);
        select_platform_integration.hint(hint);
    }

    let feedback_update = if let CollectionFormFeedback::Update {
        container_order,
        copy_link,
        description,
        download,
        embedding,
        feeds,
        offline,
        platform_integration,
        sort_number,
        title,
        unlisted,
        video_order
    } = &form_feedback {
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
        input_sort_number.validated_value(sort_number);
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        select_container_order.selected(container_order);
        select_copy_link.selected(copy_link_value);
        select_download.selected(download_value);
        select_embedding.selected(embedding_value);
        select_feeds.selected(feeds_value);
        select_platform_integration.selected(platform_integration_value);
        select_video_order.selected(video_order);
        textarea_description.value(description);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let copy_link = match collection.copy_link_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download = match collection.download_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding = match collection.embedding_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let feeds = match collection.feeds_local() {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let platform_integration = match collection.platform_integration_local() {
            Some(platform_integration) => platform_integration.key(),
            None => ""
        };

        let sort_number = collection.sort_number.as_ref().map(|number| number.to_string());

        input_offline.checked(collection.offline_local());
        input_sort_number.value_option(&sort_number);
        input_title.value_option(&collection.title);
        input_unlisted.checked(collection.unlisted_local());
        select_container_order.selected(collection.container_order.key());
        select_copy_link.selected(copy_link);
        select_download.selected(download);
        select_embedding.selected(embedding);
        select_feeds.selected(feeds);
        select_platform_integration.selected(platform_integration);
        select_video_order.selected(collection.video_order.key());
        textarea_description.value_option(&collection.description);

        String::new()
    };

    let t_upload_video_or_audio_file = translations.upload_video_or_audio_file;
    let upload_audio_video_url = routes::upload_audio_video(&collection.site_path);
    let video_upload_button = upload_widget(&upload_audio_video_url, t_upload_video_or_audio_file);

    let t_you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided = &translations.you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided;

    let new_playlist_form = {
        let mut input_slug_create_playlist = Input::permalink(
            "slug",
            &collection.site_path,
            &site,
            translations
        );
        let mut input_title_create_playlist = Input::text("title", translations.title);

        // TODO: Needs custom client-side validation - either title or slug must be given.
        input_slug_create_playlist.id("slug_playlist");
        input_title_create_playlist.id("title_playlist");

        let feedback = if let CollectionFormFeedback::CreatePlaylist { slug, title } = &form_feedback {
            input_slug_create_playlist.validated_value(slug);
            input_title_create_playlist.validated_value(title);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            String::new()
        };

        let t_create_playlist = translations.create_playlist;
        let t_new_playlist = translations.new_playlist;

        let create_playlist_action = routes::collection_create_playlist(collection);
        formatdoc!(r#"
            <div class="form_group">
                <h2>{t_new_playlist}</h2>

                <form action="{create_playlist_action}" autocomplete="off" data-warn-discard method="post">
                    <p>
                        {t_you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided}
                    </p>
                    {input_title_create_playlist}
                    {input_slug_create_playlist}
                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button class="action">{t_create_playlist}</button> {feedback}
                    </div>
                </form>
            </div>
        "#)
    };

    let new_subcollection_form = {
        let mut input_slug_create_subcollection = Input::permalink(
            "slug",
            &collection.site_path,
            &site,
            translations
        );
        let mut input_title_create_subcollection = Input::text("title", translations.title);

        // TODO: Needs custom client-side validation - either title or slug must be given.
        input_slug_create_subcollection.id("slug_subcollection");
        input_title_create_subcollection.id("title_subcollection");

        let feedback_create_subcollection = if let CollectionFormFeedback::CreateSubcollection { slug, title } = &form_feedback {
            input_slug_create_subcollection.validated_value(slug);
            input_title_create_subcollection.validated_value(title);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            String::new()
        };

        let t_create_subcollection = translations.create_subcollection;
        let t_new_subcollection = translations.new_subcollection;

        let create_subcollection_action = routes::collection_create_subcollection(collection);
        formatdoc!(r#"
            <div class="form_group">
                <h2>{t_new_subcollection}</h2>

                <form action="{create_subcollection_action}" autocomplete="off" data-warn-discard method="post">
                    <p>
                        {t_you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided}
                    </p>
                    {input_title_create_subcollection}
                    {input_slug_create_subcollection}
                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button class="action">{t_create_subcollection}</button> {feedback_create_subcollection}
                    </div>
                </form>
            </div>
        "#)
    };

    let new_video_form = {
        let mut input_slug_create_video = Input::permalink(
            "slug",
            &collection.site_path,
            &site,
            translations
        );
        let mut input_title_create_video = Input::text("title", translations.title);

        // TODO: Needs custom client-side validation - either title or slug must be given.
        input_slug_create_video.id("slug_video");
        input_title_create_video.id("title_video");

        let feedback = if let CollectionFormFeedback::CreateVideo { slug, title } = &form_feedback {
            input_slug_create_video.validated_value(slug);
            input_title_create_video.validated_value(title);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            String::new()
        };

        let t_create_video = translations.create_video;
        let t_new_video = translations.new_video;

        let create_video_action = routes::collection_create_video(collection);
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

    let input_unlisted_conditionally_hidden = if !collection.site_path.is_root_path() {
        input_unlisted.to_string()
    } else if collection.unlisted_local() {
        String::from(r#"<input name="unlisted" type="hidden" value="checked">"#)
    } else {
        String::new()
    };

    let r_resource_actions = resource_actions(
        context,
        collection.offline(),
        &site,
        &collection.site_path,
        translations
    );

    let collection_icon = hyper8_icons::collection(translations.collection);
    let collection_title = collection.title_or_slug_or_generic_label(translations);

    let t_new_video = translations.new_video;
    let t_update_collection = translations.update_collection;

    let update_action = routes::collection_edit(collection);

    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                {collection_icon}
                <span>{collection_title}</span>
            </h2>
            {r_resource_actions}
        </div>

        <div class="form_group">
            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                {input_sort_number}
                {select_video_order}
                {select_container_order}
                {select_copy_link}
                {select_download}
                {select_embedding}
                {select_feeds}
                {select_platform_integration}
                {input_offline}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_collection}</button> {feedback_update}
                </div>
            </form>
        </div>

        {new_subcollection_form}

        {new_playlist_form}

        {new_video_form}

        <div class="form_group">
            <h2>{t_new_video}</h2>
            {video_upload_button}
        </div>
    "#);

    let t_collection_settings = translations.collection_settings;
    let collection_title = collection.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_collection_settings} – {collection_title}");
    let html = layout(
        &body,
        context,
        Location::edit(&collection.site_path),
        session_state,
        &site,
        &head_title
    );

    html_response(html)
}

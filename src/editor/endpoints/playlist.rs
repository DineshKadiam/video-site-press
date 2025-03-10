// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;

use indoc::formatdoc;

use crate::{
    Context,
    Field,
    Language,
    PlatformIntegration,
    Playlist
};
use crate::editor::{Checkbox, Input, Select, Textarea};
use crate::editor::endpoints;
use crate::editor::widgets::{
    add_link_form,
    layout,
    links,
    resource_actions,
    upload_widget
};
use crate::icons;

mod add_link;
mod create_video;
mod delete;
mod delete_banner;
mod delete_link;
mod edit;
mod r#move;
mod update;
mod update_banner;
mod upload_banner;

pub use add_link::add_link;
pub use create_video::create_video;
pub use delete::delete;
pub use delete_banner::delete_banner;
pub use delete_link::delete_link;
pub use edit::edit;
pub use r#move::r#move;
pub use update::update;
pub use update_banner::update_banner;
pub use upload_banner::upload_banner;

pub enum PlaylistFormFeedback<'a> {
    CreateVideo {
        slug: Field,
        title: Field
    },
    Move {
        slug: Field
    },
    Site(endpoints::site::SiteFormFeedback),
    Update {
        copy_link: Option<bool>,
        description: &'a str,
        download: Option<bool>,
        embedding: Option<bool>,
        offline: bool,
        order: &'a str,
        platform_integration: Option<PlatformIntegration>,
        title: Field,
        unlisted: bool
    }
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<PlaylistFormFeedback>,
    language: &Language,
    playlist: &Playlist
) -> HttpResponse {
    let site = context.get_site();
    let site_path_normalized = playlist.site_path.normalized();
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
    let banner_upload_button = upload_widget(&format!("/upload-playlist-banner/{site_path_normalized}"), t_upload_banner);
    let banner_form = match &playlist.banner {
        Some(banner) => {
            let alt = if let Some(description) = &banner.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            // TODO: Could/should also use computed assets (also elsewhere, videos etc.)
            let file_name = &banner.file_name;
            let src = format!("/poster/{}{file_name}", playlist.site_path.normalized());

            let mut description = Textarea::image_description("description", translations.banner_description);

            description
                .id("banner_description")
                .value_option(&banner.description);

            let t_delete_banner = translations.delete_banner;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;
            format!(r#"
                <img {alt} class="thumbnail" src="{src}">
                <div class="button_group">
                    {banner_upload_button}
                    <form action="/delete-playlist-banner/{site_path_normalized}" data-confirm data-confirm-label="{t_delete_banner}" data-confirm-text="{t_really_delete}" method="post">
                        <button class="danger">{t_delete_banner}</button>
                    </form>
                </div>
                <form action="/update-playlist-banner/{site_path_normalized}" data-warn-discard method="post">
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
    let mut select_order = Select::new("order", translations.order);
    let mut select_platform_integration = Select::new("platform_integration", translations.automatic_link_previews_and_content_embedding_on_platforms);
    let mut textarea_description = Textarea::markdown_subset(language, "description", translations.description);

    let t_disabled = translations.disabled;
    let t_enabled = translations.enabled;
    let t_inherit = translations.inherit;

    let t_copy_link_disabled_enabled = if playlist.copy_link_inherited() { t_enabled } else { t_disabled };
    select_copy_link.option("", format!("{t_inherit} ({t_copy_link_disabled_enabled})"));
    select_copy_link.option("enabled", t_enabled);
    select_copy_link.option("disabled", t_disabled);

    let t_download_disabled_enabled = if playlist.download_inherited() { t_enabled } else { t_disabled };
    select_download.option("", format!("{t_inherit} ({t_download_disabled_enabled})"));
    select_download.option("enabled", t_enabled);
    select_download.option("disabled", t_disabled);

    let t_embedding_disabled_enabled = if playlist.embedding_inherited() { t_enabled } else { t_disabled };
    select_embedding.option("", format!("{t_inherit} ({t_embedding_disabled_enabled})"));
    select_embedding.option("enabled", t_enabled);
    select_embedding.option("disabled", t_disabled);

    select_order.option("default", translations.default_sort_number_or_release_date_ascending);
    select_order.option("release_date_asc", translations.release_date_ascending);
    select_order.option("release_date_desc", translations.release_date_descending);
    select_order.option("sort_number_asc", translations.sort_number_ascending);
    select_order.option("sort_number_desc", translations.sort_number_descending);
    select_order.option("title_asc", translations.title_ascending);
    select_order.option("title_desc", translations.title_descending);

    let platform_integration_inherited = playlist.platform_integration_inherited();
    let t_platform_integration_inherited_label = platform_integration_inherited.label(translations);
    select_platform_integration.option("", format!("{t_inherit} ({t_platform_integration_inherited_label})"));
    select_platform_integration.option("disabled", t_disabled);
    select_platform_integration.option("link_previews", translations.link_previews_only);
    select_platform_integration.option("content_embeds", translations.link_previews_and_content_embeds);

    textarea_description.id("playlist_description");

    let feedback_update = if let Some(PlaylistFormFeedback::Update {
        copy_link,
        description,
        download,
        embedding,
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
        select_order.selected(playlist.order.key());
        select_platform_integration.selected(platform_integration);
        textarea_description.value_option(&playlist.description);

        String::new()
    };

    let slug_form = if playlist.site_path.is_root_path() {
        String::new()
    } else {
        let mut input_permalink = Input::permalink(
            "slug",
            &playlist.parent_site_path_unchecked(),
            &site,
            translations
        );

        input_permalink.required();

        let feedback = if let Some(PlaylistFormFeedback::Move { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(playlist.slug_unchecked());
            String::new()
        };

        let t_a_permalink_forms_part_of_text = translations.a_permalink_forms_part_of_text;
        let t_update_permalink = translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_forms_part_of_text}
            </p>

            <form action="/move-playlist/{site_path_normalized}" data-warn-discard method="post">
                {input_permalink}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="danger">{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let t_upload_new_video = translations.upload_new_video;
    let upload_url = format!("/upload-video/{site_path_normalized}");
    let video_upload_button = upload_widget(&upload_url, t_upload_new_video);

    let site_form_feedback = match &form_feedback {
        Some(PlaylistFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(
        context,
        site_form_feedback,
        &playlist.site_path,
        translations
    );

    let t_convert_to_collection = translations.convert_to_collection;
    let convert_to_collection_form = formatdoc!(r#"
        <p>
            A playlist can seamlessly be converted to a collection
        </p>

        <form action="/convert-to-collection/{site_path_normalized}" method="post">
            <button class="danger">{t_convert_to_collection}</button>
        </form>
    "#);

    let convert_to_video_form = if playlist.videos.len() < 2 {
        let message = if playlist.videos.is_empty() {
            "This playlist is empty and can be seamlessly converted to a video."
        } else {
            "This playlist contains a single video and can be converted to just a video, note however that playlist data will be removed or overwritten with data from the video where necessary."
        };

        let t_convert_to_video = translations.convert_to_video;
        formatdoc!(r#"
            <p>{message}</p>

            <form action="/convert-to-video/{site_path_normalized}" method="post">
                <button class="danger">{t_convert_to_video}</button>
            </form>
        "#)
    } else {
        String::new()
    };

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
        formatdoc!(r#"
            <div class="form_group">
                <h2>{t_new_video}</h2>

                <form action="/create-playlist-video/{site_path_normalized}" autocomplete="off" data-warn-discard method="post">
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

    let create_link_form = add_link_form(&format!("/add-playlist-link/{site_path_normalized}"), translations);
    let links_form = links(&playlist.links, &format!("/delete-playlist-link/{site_path_normalized}"), translations);

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
    let t_danger_zone = translations.danger_zone;
    let t_delete_playlist = translations.delete_playlist;
    let t_deleting_the_playlist_text = translations.deleting_the_playlist_text;
    let t_new_video = translations.new_video;
    let t_really_delete = translations.really_delete;
    let t_update_playlist = translations.update_playlist;
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
            <form action="/playlist/{site_path_normalized}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                {select_order}
                <div class="form_split even">
                    {select_copy_link}
                    {select_download}
                    {select_embedding}
                </div>
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
            <p style="background: var(--danger); border-radius: .5rem; color: var(--light-1); padding: 1rem;">
                Heads up: Banner images are not yet used in the layout,<br>but this will be included in a future version of Hyper 8
            </p>
            {banner_form}
        </div>

        {site_edit}

        <div class="form_group">
            <h2>{t_danger_zone}</h2>

            {slug_form}

            {convert_to_collection_form}

            {convert_to_video_form}

            <p>
                {t_deleting_the_playlist_text}
            </p>

            <form
                action="/delete-playlist/{site_path_normalized}"
                data-confirm
                data-confirm-label="{t_delete_playlist}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_playlist}</button>
            </form>
        </div>
    "#);

    let html = layout(
        &body,
        context,
        language,
        &site,
        Some(&playlist.site_path),
        playlist.title_or_slug_or_generic_label(translations)
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

// pub async fn create(
//     context: Data<Arc<Context>>,
//     form: Form<PlaylistCreateForm>
// ) -> Either<Redirect, HttpResponse> {
//     // TODO: Show errors when re-rendering edit/create form
//     if !form.title.trim().is_empty() {
//         let mut playlist = Playlist::new(&form.title);

//         let (playlist, playlists) = {
//             let mut playlists_mut = context.collection.playlists.lock().unwrap();

//             while playlists_mut.iter().any(|iterated| iterated.slug == playlist.slug) {
//                 // Create a modified, unique slug
//                 playlist.slug = format!("{}_{}", &playlist.slug, nanoid());
//             }

//             playlists_mut.push(playlist.clone());
            
//             (playlist, playlists_mut.clone())
//         };

//         playlist.persist_in_cache(&context.cache_dir);

//         // This persists them in the collection
//         persist_playlists(&playlists, &context.site_dir);

//         let redirect_url = format!("/playlist/{}", playlist.id);
//         Either::Left(
//             Redirect::to(redirect_url).see_other()
//         )
//     } else {
//         // TODO: Handle empty title - how do we respond? (note that this is an edge edge edge case)
//         panic!();

//         // let settings = context.collection.get_settings();
//         // let html = layout(&settings, &body, t_new_playlist);

//         // Either::Right(
//         //     HttpResponse::Ok()
//         //         .content_type(ContentType::html())
//         //         .body(html)
//         // )
//     }
// }

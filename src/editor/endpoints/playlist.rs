// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    Banner,
    Context,
    ContextPath,
    Field,
    FileMeta,
    HyperUrl,
    JobKind,
    Language,
    Link,
    Playlist,
    PlaylistOrder
};
use crate::editor::{Checkbox, Input, Select, Textarea};
use crate::editor::endpoints;
use crate::editor::widgets::{
    add_link_form,
    error_page,
    layout,
    links,
    not_found,
    path_widget,
    preview_button,
    upload_widget
};
use crate::icons;
use crate::util::{nanoid, supported_image_extension, validate_slug};

#[derive(Deserialize)]
pub struct AddLinkForm {
    label: String,
    url: String,
    verification: String
}

#[derive(Deserialize)]
pub struct BannerUpdateForm {
    description: String
}

#[derive(Deserialize)]
pub struct DeleteLinkForm {
    label: String,
    url: String
}

pub enum PlaylistFormFeedback<'a> {
    Move {
        slug: Field
    },
    Site(endpoints::site::SiteFormFeedback),
    Update {
        description: &'a str,
        order: &'a str,
        title: Field,
        unlisted: bool
    }
}

#[derive(Deserialize)]
pub struct PlaylistMoveForm {
    slug: String
}

#[derive(Deserialize)]
pub struct PlaylistUpdateForm {
    description: String,
    order: String,
    title: String,
    unlisted: Option<String>
}

pub async fn add_link(
    context: Data<Arc<Context>>,
    form: Form<AddLinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let (hidden, rel_me) = match form.verification.as_str() {
        "rel-me" => (false, true),
        "rel-me-hidden" => (true, true),
        _ => (false, false)
    };
    let label_trimmed = form.label.trim();
    let label = match label_trimmed.is_empty() {
        true => None,
        false => Some(label_trimmed.to_string())
    };
    let url = match HyperUrl::parse(&form.url) {
        Ok(url) => url,
        Err(err) => {
            let language = context.get_editor_language();
            let t_invalid_url = language.translations.invalid_url;
            let message = format!("{t_invalid_url} ({err})");
            return Either::Right(error_page(&context, &language, &message))
        }
    };

    let mutation = |playlist_mut: &mut Playlist| {
        let link = Link::new(hidden, label, rel_me, url);
        playlist_mut.links.push(link);
        playlist_mut.write_manifest(&context);
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_playlist(&path.path) {
        let language = context.get_editor_language();
        return Either::Right(not_found(&context, &language, &path.path))
    };

    let redirect_url = if &path.path == "" {
        "/".to_string()
    } else {
        // We join("") to append a trailing slash (which parent() removes beforehand)
        Path::new("/collection/").join(&path.path).parent().unwrap().join("").to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub async fn delete_link(
    context: Data<Arc<Context>>,
    form: Form<DeleteLinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if playlist_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url.representation == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url.representation == form.url
                }
            )
            .map(|index| playlist_mut.links.remove(index))
            .is_some() {
            playlist_mut.write_manifest(&context);
        }
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn delete_banner(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if let Some(banner) = &playlist_mut.banner {
            let banner_path = context.site_dir.join(&playlist_mut.path).join(&banner.file_name);
            playlist_mut.banner = None;
            playlist_mut.write_manifest(&context);
            let _ = fs::remove_file(banner_path);
        }
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
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

    let path = &playlist.path;

    let t_upload_banner = translations.upload_banner;
    let banner_upload_button = upload_widget(&format!("/upload-playlist-banner/{path}"), t_upload_banner);
    let banner_form = match &playlist.banner {
        Some(banner) => {
            let alt = if let Some(description) = &banner.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            // TODO: Could/should also use computed assets (also elsewhere, videos etc.)
            let file_name = &banner.file_name;
            let src = if path == "" {
                format!("/poster/{file_name}")
            } else {
                format!("/poster/{path}/{file_name}")
            };

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
                    <form action="/delete-playlist-banner/{path}" data-confirm data-confirm-label="{t_delete_banner}" data-confirm-text="{t_really_delete}" method="post">
                        <button class="danger">{t_delete_banner}</button>
                    </form>
                </div>
                <form action="/update-playlist-banner/{path}" data-warn-discard method="post">
                    {description}
                    <button class="action">{t_update_description}</button>
                </form>
            "#)
        }
        None => banner_upload_button
    };

    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut select_order = Select::new("order", translations.order);
    let mut textarea_description = Textarea::markdown_subset(language, "description", translations.description);

    select_order.option("default", translations.default_sort_number_or_release_date_ascending);
    select_order.option("release_date_asc", translations.release_date_ascending);
    select_order.option("release_date_desc", translations.release_date_descending);
    select_order.option("sort_number_asc", translations.sort_number_ascending);
    select_order.option("sort_number_desc", translations.sort_number_descending);
    select_order.option("title_asc", translations.title_ascending);
    select_order.option("title_desc", translations.title_descending);

    textarea_description.id("playlist_description");

    let feedback_update = if let Some(PlaylistFormFeedback::Update { description, order, title, unlisted }) = &form_feedback {
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        select_order.selected(order);
        textarea_description.value(description);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        input_title.value_option(&playlist.title);
        input_unlisted.checked(playlist.unlisted);
        select_order.selected(playlist.order.key());
        textarea_description.value_option(&playlist.description);

        String::new()
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let mut input_permalink = Input::permalink("slug", translations.permalink);

        input_permalink.required();

        let feedback = if let Some(PlaylistFormFeedback::Move { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(playlist.slug());
            String::new()
        };

        let t_a_permalink_defines_under_which_url_text = translations.a_permalink_defines_under_which_url_text;
        let t_update_permalink = translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_defines_under_which_url_text}
            </p>

            <form action="/move-playlist/{path}" data-warn-discard method="post">
                {input_permalink}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="danger">{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let t_upload_new_video = translations.upload_new_video;
    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, t_upload_new_video);

    let site_form_feedback = match &form_feedback {
        Some(PlaylistFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, path, translations);

    let t_convert_to_collection = translations.convert_to_collection;
    let convert_to_collection_form = formatdoc!(r#"
        <p>
            A playlist can seamlessly be converted to a collection
        </p>

        <form action="/convert-to-collection/{path}" method="post">
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

            <form action="/convert-to-video/{path}" method="post">
                <button class="danger">{t_convert_to_video}</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let input_unlisted_conditionally_hidden = match playlist.path.as_str() {
        "" => if playlist.unlisted { format!(r#"<input name="unlisted" type="hidden" value="checked">"#)} else { String::new() },
        _ => input_unlisted.to_string()
    };

    let create_link_form = add_link_form(&format!("/add-playlist-link/{path}"), translations);
    let links_form = links(&playlist.links, &format!("/delete-playlist-link/{path}"), translations);

    let r_path_widget = path_widget(context, path, &site, translations);
    let r_preview_button = preview_button(path, translations);

    let playlist_icon = icons::playlist(translations.playlist);
    let playlist_title = playlist.label(translations);

    let t_banner_image = translations.banner_image;
    let t_danger_zone = translations.danger_zone;
    let t_delete_playlist = translations.delete_playlist;
    let t_deleting_the_playlist_text = translations.deleting_the_playlist_text;
    let t_locations = translations.locations;
    let t_new_video = translations.new_video;
    let t_really_delete = translations.really_delete;
    let t_update_playlist = translations.update_playlist;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                {playlist_icon}
                <span>{playlist_title}</span>
                {r_preview_button}
            </h2>

            <form action="/playlist/{path}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                {select_order}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_playlist}</button> {feedback_update}
                </div>
            </form>
        </div>

        <div class="form_group">
            <h2>{t_locations}</h2>
            {r_path_widget}
        </div>

        {links_form}

        {create_link_form}

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
                action="/delete-playlist/{path}"
                data-confirm
                data-confirm-label="{t_delete_playlist}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_playlist}</button>
            </form>
        </div>
    "#);

    let head_title = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
    let html = layout(context, language, Some(path), &site, &body, head_title);

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

pub async fn edit(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_playlist(&path.path) {
        Some(playlist) => edit_page(&context, None, &language, &playlist),
        None => not_found(&context, &language, &path.path)
    }
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<PlaylistMoveForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let new_path = Path::new(&path.path)
        .with_file_name(slug_trimmed)
        .into_os_string()
        .into_string()
        .unwrap();

    // TODO: Here and elsewhere: If a playlist already has a permalink (say "example"),
    //       and we try to save it with exactly the same/unchanged permalink "example",
    //       we get an error that *another* playlist is already using that permalink.
    //       Need to make an exception for this in the validation check.
    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&new_path).exists() {
        Some(format!("This permalink is already used by another collection, playlist or video."))
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |playlist: &mut Playlist| {
            fs::rename(
                context.site_dir.join(&playlist.path),
                context.site_dir.join(&new_path)
            ).unwrap();

            playlist.path = new_path;
        };

        let playlist = match context.update_playlist(mutation, &path.path) {
            Ok(playlist) => playlist,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = PlaylistFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &playlist)
        )
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<PlaylistUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let description_trimmed = form.description.trim();
    let order = PlaylistOrder::from_key(&form.order).unwrap();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |playlist: &mut Playlist| {
            playlist.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            playlist.order = order;
            playlist.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            playlist.unlisted = unlisted;
            playlist.write_manifest(&context);
        };

        let playlist = match context.update_playlist(mutation, &path.path) {
            Ok(playlist) => playlist,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = PlaylistFormFeedback::Update {
            description: description_trimmed,
            order: &form.order,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &playlist)
        )
    }
}

pub async fn update_banner(
    context: Data<Arc<Context>>,
    form: Form<BannerUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if let Some(banner) = &mut playlist_mut.banner {
            banner.description = Some(form.description.clone());
        }
        playlist_mut.write_manifest(&context);
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn upload_banner(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(file_name) = content_disposition.get_filename() {
            if !supported_image_extension(file_name) {
                let message = language.translations.unsupported_file_extension_in_xxx(file_name);
                return Ok(Either::Right(error_page(&context, &language, &message)))
            }

            let original_filename = file_name.to_string();
            // TODO: Generate a prettier filename - this goes into the site_dir!
            //       1) original_filename (if not taken) 2) only otherwise add nanoid component to filename
            let storage_filename = format!("{}_{original_filename}", nanoid());
            let storage_path = context.site_dir.join(&path.path).join(&storage_filename);
            let storage_path_moving = storage_path.clone();

            // File::create is blocking operation, use threadpool
            let mut f = web::block(|| File::create(storage_path_moving)).await??;

            while let Some(chunk) = field.try_next().await? {
                f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
            }

            let file_meta = FileMeta::new(&storage_path);

            // TODO: Although unlikely to result in anything, most of the time,
            //       we could fetch assets from the cache, just in case?

            let banner = Banner::new(None, None, file_meta, storage_filename);

            let mutation = |playlist_mut: &mut Playlist| {
                playlist_mut.banner = Some(banner);
                playlist_mut.write_manifest(&context);
            };

            match context.update_playlist(mutation, &path.path) {
                Ok(playlist) => {
                    let job_kind = JobKind::PlaylistFastTasks(Arc::clone(&playlist));
                    context.push_job_queue(job_kind);
                }
                Err(()) => {
                    let _ = fs::remove_file(&storage_path);
                    let http_response = not_found(&context, &language, &path.path);
                    return Ok(Either::Right(http_response));
                }
            }
        }
    }

    let redirect_url = format!("/playlist/{}", path.path);

    Ok(Either::Left(Redirect::to(redirect_url).see_other()))
}
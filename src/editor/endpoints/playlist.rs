use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{self, Data, Form, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    Banner,
    Context,
    ContextPath,
    Field,
    FileMeta,
    JobKind,
    Playlist
};
use crate::editor::endpoints;
use crate::editor::widgets::{
    form_checkbox,
    form_field,
    form_field_pattern,
    form_textarea,
    layout,
    not_found,
    upload_widget
};
use crate::util::{nanoid, supported_image_extension, validate_slug};

#[derive(Deserialize)]
pub struct BannerUpdateForm {
    description: String
}

pub enum PlaylistFormFeedback<'a> {
    Move {
        slug: Field
    },
    Site(endpoints::site::SiteFormFeedback),
    Update {
        description: &'a str,
        title: Field,
        unlisted: bool
    }
}

// #[derive(Deserialize)]
// pub struct PlaylistCreateForm {
//     title: String
// }

#[derive(Deserialize)]
pub struct PlaylistMoveForm {
    slug: String
}

#[derive(Deserialize)]
pub struct PlaylistUpdateForm {
    description: String,
    title: String,
    unlisted: Option<String>
}

#[derive(Deserialize)]
pub struct VideoForm {
    slug: String
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_playlist(&path.path) {
        return Either::Right(not_found(&context, &format!("Playlist {} not found.", &path.path)))
    };

    let redirect_url = if &path.path == "" {
        "/".to_string()
    } else {
        Path::new("/collection").join(&path.path).parent().unwrap().to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub async fn delete_banner(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist: &mut Playlist| {
        if let Some(banner) = &playlist.banner {
            let banner_path = context.site_dir.join(&playlist.path).join(&banner.file_name);
            playlist.banner = None;
            playlist.write_manifest(&context.site_dir);
            let _ = fs::remove_file(banner_path);
        }
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => Either::Right(
            not_found(&context, &format!("Playlist {} not found.", &path.path))
        )
    }
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<PlaylistFormFeedback>,
    playlist: &Playlist
) -> HttpResponse {
    let t_not_saved = &context.editor_translations.not_saved;

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

    let t_delete_banner = &context.editor_translations.delete_banner;
    let t_upload_banner = &context.editor_translations.upload_banner;
    let banner_upload_button = upload_widget(&format!("/upload-banner/{path}"), t_upload_banner);
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

            let t_banner_description = &context.editor_translations.banner_description;
            let description = form_textarea("description", t_banner_description, banner.description.as_deref());

            let t_update_description = &context.editor_translations.update_description;
            format!(r#"
                <img {alt} class="thumbnail" src="{src}">
                <div class="button_group">
                    {banner_upload_button}
                    <form action="/delete-banner/{path}" method="post">
                        <button>{t_delete_banner}</button>
                    </form>
                </div>
                <form action="/update-banner/{path}" method="post">
                    {description}
                    <button>{t_update_description}</button>
                </form>
            "#)
        }
        None => banner_upload_button
    };

    let t_description = &context.editor_translations.description;
    let t_title = &context.editor_translations.title;
    let t_unlisted = &context.editor_translations.unlisted;
    let (
        feedback_update,
        input_description,
        input_title,
        input_unlisted,
    ) = if let Some(PlaylistFormFeedback::Update { description, title, unlisted }) = &form_feedback {
        (
            format!(r#"<span class="feedback">{t_not_saved}</span>"#),
            form_textarea("description", t_description, Some(description)),
            form_field(title, "title", t_title),
            form_checkbox(*unlisted, "unlisted", t_unlisted)
        )
    } else {
        (
            String::new(),
            form_textarea("description", t_description, playlist.description.as_deref()),
            form_field(&Field::valid(playlist.title.as_deref().unwrap_or("")), "title", t_title),
            form_checkbox(playlist.unlisted, "unlisted", t_unlisted)
        )
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let (input, feedback) = if let Some(PlaylistFormFeedback::Move { slug }) = &form_feedback {
            (
                // TODO: Double-check this pattern (and elsewhere checks from start to end ($^?))
                form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                format!(r#"<span class="feedback">{t_not_saved}</span>"#)
            )
        } else {
            (
                form_field_pattern(&Field::valid(playlist.slug()), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                String::new()
            )
        };

        let t_update_permalink = &context.editor_translations.update_permalink;
        formatdoc!(r#"
            <p>
                Be aware that changing the slug/permalink of a playlist changes
                the url under which the playlist is publicly available on the web,
                or in other words, if there are links pointing to your playlist
                right now, they will not work anymore when you change the slug.
            </p>

            <form action="/move-playlist/{path}" method="post">
                {input}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let t_upload_new_video = &context.editor_translations.upload_new_video;
    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, t_upload_new_video);

    let site_form_feedback = match &form_feedback {
        Some(PlaylistFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, path);

    let convert_to_collection_form = formatdoc!(r#"
        <p>
            A playlist can seamlessly be converted to a collection, but note that the
            banner will be deleted, because collections don't have one.
        </p>

        <form action="/convert-to-collection/{path}" method="post">
            <button>Convert playlist to collection</button>
        </form>
    "#);

    let convert_to_video_form = if playlist.videos.len() < 2 {
        let message = if playlist.videos.is_empty() {
            "This playlist is empty and can be seamlessly converted to a video."
        } else {
            "This playlist contains a single video and can be converted to just a video, note however that playlist data will be removed or overwritten with data from the video where necessary."
        };

        formatdoc!(r#"
            <p>{message}</p>

            <form action="/convert-to-video/{path}" method="post">
                <button>Convert playlist to video</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let input_unlisted_conditionally_hidden = match playlist.path.as_str() {
        "" => if playlist.unlisted { format!(r#"<input name="unlisted" type="hidden" value="checked">"#)} else { String::new() },
        _ => input_unlisted
    };

    let t_banner_image = &context.editor_translations.banner_image;
    let t_danger_zone = &context.editor_translations.danger_zone;
    let t_delete_playlist = &context.editor_translations.delete_playlist;
    let t_new_video = &context.editor_translations.new_video;
    let t_playlist = &context.editor_translations.playlist;
    let t_really_delete = &context.editor_translations.really_delete;
    let t_update_playlist = &context.editor_translations.update_playlist;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2>{t_playlist}</h2>

            <form action="/playlist/{path}" method="post">
                {input_title}
                {input_description}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_playlist}</button> {feedback_update}
                </div>
            </form>
        </div>

        <div class="form_group">
            <h2>{t_new_video}</h2>

            {video_upload_button}
        </div>

        <div class="form_group">
            <h2>{t_banner_image}</h2>
            {banner_form}
        </div>

        {site_edit}

        <div class="form_group">
            <h2>{t_danger_zone}</h2>

            {slug_form}

            {convert_to_collection_form}

            {convert_to_video_form}

           <p>
                Deleting the playlist removes the playlist manifest, banner image,
                and all videos contained in the playlist. If the playlist lies at the
                site root it does not remove the site manifest and the site itself,
                so the site can afterwards be reinitialized again as a collection,
                playlist or video.
            </p>

            <form action="/delete-playlist/{path}" method="post">
                <button onclick="if (!confirm('{t_really_delete}')) {{ return false; }}">{t_delete_playlist}</button>
            </form>
        </div>
    "#);

    let site = context.get_site();
    let head_title = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
    let html = layout(context, Some(path), &site, &body, head_title);

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
    // TODO: Should be cloned because the returned one is mut and locking?
    let playlist = match context.get_playlist(&path.path) {
        Some(playlist) => playlist,
        None => return not_found(&context, &format!("Playlist {} not found.", &path.path))
    };

    edit_page(&context, None, &playlist)
}

pub async fn remove_video(
    context: Data<Arc<Context>>,
    form: Form<VideoForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !form.slug.is_empty() {
        let mutation = |playlist: &mut Playlist| {
            if let Some(index) = playlist.videos_virtual
                .iter()
                .position(|slug| slug == &form.slug) {
                playlist.videos_virtual.remove(index);
                playlist.write_manifest(&context.site_dir);
            }
        };

        let playlist = match context.update_playlist(mutation, &path.path) {
            Ok(playlist) => playlist,
            Err(()) => return Either::Right(not_found(&context, &format!("Playlist {} not found.", &path.path)))
        };

        let redirect_url = format!("/playlist/{}", playlist.path);

        Either::Left(
            Redirect::to(redirect_url).see_other()
        )
    } else {
        // TODO: Implement the entire thing properly
        panic!()
    }
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<PlaylistMoveForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let slug_trimmed = form.slug.trim();

    let new_path = Path::new(&path.path)
        .with_file_name(slug_trimmed)
        .into_os_string()
        .into_string()
        .unwrap();

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
            Err(()) => return Either::Right(
                not_found(&context, &format!("Playlist {} not found.", &path.path))
            )
        };

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(
                not_found(&context, &format!("Playlist {} not found.", &path.path))
            )
        };

        let form_feedback = PlaylistFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &playlist)
        )
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<PlaylistUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let description_trimmed = form.description.trim();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |playlist: &mut Playlist| {
            playlist.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            playlist.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            playlist.unlisted = unlisted;
            playlist.write_manifest(&context.site_dir);
        };

        let playlist = match context.update_playlist(mutation, &path.path) {
            Ok(playlist) => playlist,
            Err(()) => return Either::Right(
                not_found(&context, &format!("Playlist {} not found.", &path.path))
            )
        };

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(
                not_found(&context, &format!("Playlist {} not found.", &path.path))
            )
        };

        let form_feedback = PlaylistFormFeedback::Update {
            description: description_trimmed,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &playlist)
        )
    }
}

pub async fn update_banner(
    context: Data<Arc<Context>>,
    form: Form<BannerUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist: &mut Playlist| {
        if let Some(banner) = &mut playlist.banner {
            banner.description = Some(form.description.clone());
        }
        playlist.write_manifest(&context.site_dir);
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => return Either::Right(
            not_found(&context, &format!("Playlist {} not found.", &path.path))
        )
    }
}

pub async fn upload_banner(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            if !supported_image_extension(filename) {
                // TODO: Implement without panicking - return something instead
                panic!("Unsupported file extension in {filename}");
            }

            let original_filename = filename.to_string();
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

            let mutation = |playlist: &mut Playlist| {
                playlist.banner = Some(banner);
                playlist.write_manifest(&context.site_dir);
            };

            match context.update_playlist(mutation, &path.path) {
                Ok(playlist) => {
                    let job_kind = JobKind::PlaylistFastTasks(Arc::clone(&playlist));
                    context.push_job_queue(job_kind);
                }
                Err(()) => {
                    let _ = fs::remove_file(&storage_path);
                    let http_response = not_found(&context, &format!("Playlist {} not found.", &path.path));
                    return Ok(Either::Right(http_response));
                }
            }
        }
    }

    let redirect_url = format!("/playlist/{}", path.path);

    Ok(Either::Left(Redirect::to(redirect_url).see_other()))
}
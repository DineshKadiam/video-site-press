use std::fs;
use std::path::Path;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Either, Form, Path as ActixPath, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;
use url::Url;

use crate::{
    Collection,
    Context,
    ContextPath,
    Field,
    Language,
    Link,
    Playlist
};
use crate::editor::endpoints;
use crate::editor::widgets::{
    error_page,
    form_checkbox,
    form_field,
    form_field_pattern,
    form_textarea,
    layout,
    not_found,
    upload_widget
};
use crate::icons;
use crate::util::validate_slug;

pub enum CollectionFormFeedback<'a> {
    CreatePlaylist {
        slug: Field
    },
    CreateSubcollection {
        slug: Field
    },
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

#[derive(Deserialize)]
pub struct LinkForm {
    label: String,
    url: String
}

#[derive(Deserialize)]
pub struct SlugForm {
    slug: String
}

#[derive(Deserialize)]
pub struct UpdateForm {
    description: String,
    title: String,
    unlisted: Option<String>
}

pub async fn add_link(
    context: Data<Arc<Context>>,
    form: Form<LinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let url = match Url::parse(&form.url) {
        Ok(url) => url,
        Err(err) => {
            // TODO: Localize (= add our own message to/instead of the parsing error from Url crate)
            let language = context.get_editor_language();
            return Either::Right(error_page(&context, &language, &err.to_string()))
        }
    };

    let mutation = |collection_mut: &mut Collection| {
        let link = Link::new(form.label.clone(), url);
        collection_mut.links.push(link);
        collection_mut.write_manifest(&context);
    };

    match context.update_collection(mutation, &path.path) {
        Ok(collection) => {
            let redirect_url = format!("/collection/{}", collection.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn create_playlist(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let playlist_path = if &path.path == "" {
        slug_trimmed.to_string()
    } else {
        format!("{}/{slug_trimmed}", &path.path)
    };

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&playlist_path).exists() {
        Some("Already used by another collection, playlist or video".to_string())
    } else {
        None
    };

    if slug_error.is_none() {
        let playlist_dir = context.site_dir.join(&playlist_path);
        fs::create_dir_all(&playlist_dir).unwrap();

        let playlist = Arc::new(Playlist::new(playlist_path));

        let playlist_moving = Arc::clone(&playlist);
        let mutation = |collection: &mut Collection| {
            collection.playlists.push(playlist_moving);
        };

        if context.update_collection(mutation, &path.path).is_err() {
            return Either::Right(not_found(&context, &language, &path.path));
        }

        playlist.write_manifest(&context);

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::CreatePlaylist {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback), &language))
    }
}

pub async fn create_subcollection(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let subcollection_path = if &path.path == "" {
        slug_trimmed.to_string()
    } else {
        format!("{}/{slug_trimmed}", &path.path)
    };

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&subcollection_path).exists() {
        Some("Already used by another collection, playlist or video".to_string())
    } else {
        None
    };

    if slug_error.is_none() {
        let subcollection_dir = context.site_dir.join(&subcollection_path);
        fs::create_dir_all(&subcollection_dir).unwrap();

        let subcollection = Arc::new(Collection::new(subcollection_path));

        let subcollection_moving = Arc::clone(&subcollection);
        let mutation = |collection: &mut Collection| {
            collection.subcollections.push(subcollection_moving);
        };

        if context.update_collection(mutation, &path.path).is_err() {
            return Either::Right(not_found(&context, &language, &path.path));
        }

        subcollection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", subcollection.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::CreateSubcollection {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback), &language))
    }
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_collection(&path.path) {
        let language = context.get_editor_language();
        return Either::Right(not_found(&context, &language, &path.path))
    };

    let redirect_url = if &path.path == "" {
        "/".to_string()
    } else {
        Path::new("/collection").join(&path.path).parent().unwrap().to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub async fn delete_link(
    context: Data<Arc<Context>>,
    form: Form<LinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |collection: &mut Collection| {
        if let Ok(url) = Url::parse(&form.url) {
            if collection.links
                .iter()
                .position(|link| link.label == form.label && link.url == url)
                .map(|index| collection.links.remove(index))
                .is_some() {
                collection.write_manifest(&context);
            }
        }
    };

    match context.update_collection(mutation, &path.path) {
        Ok(collection) => {
            let redirect_url = format!("/collection/{}", collection.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub fn edit_page(
    collection: &Collection,
    context: &Context,
    form_feedback: Option<CollectionFormFeedback>,
    language: &Language
) -> HttpResponse {
    let path = &collection.path;
    let t_not_saved = language.translations.not_saved;

    let errors = if collection.errors.is_empty() {
        String::new()
    } else {
        collection.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let t_description = language.translations.description;
    let t_title = language.translations.title;
    let t_unlisted = language.translations.unlisted;
    let (
        feedback_update,
        input_description,
        input_title,
        input_unlisted,
    ) = if let Some(CollectionFormFeedback::Update { description, title, unlisted }) = &form_feedback {
        (
            format!(r#"<span class="feedback">{t_not_saved}</span>"#),
            form_textarea("description", t_description, Some(description)),
            form_field(title, "title", t_title),
            form_checkbox(*unlisted, "unlisted", t_unlisted)
        )
    } else {
        (
            String::new(),
            form_textarea("description", t_description, collection.description.as_deref()),
            form_field(&Field::valid(collection.title.as_deref().unwrap_or("")), "title", t_title),
            form_checkbox(collection.unlisted, "unlisted", t_unlisted)
        )
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let (input, feedback) = if let Some(CollectionFormFeedback::Move { slug }) = &form_feedback {
            (
                form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                format!(r#"<span class="feedback">{t_not_saved}</span>"#)
            )
        } else {
            (
                form_field_pattern(&Field::valid(collection.slug()), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                String::new()
            )
        };

        let t_a_permalink_defines_under_which_url_text = language.translations.a_permalink_defines_under_which_url_text;
        let t_update_permalink = language.translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_defines_under_which_url_text}
            </p>

            <form action="/move-collection/{path}" method="post">
                {input}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let t_upload_new_video = language.translations.upload_new_video;
    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, t_upload_new_video);

    let site_form_feedback = match &form_feedback {
        Some(CollectionFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, path, &language.translations);

    let (input_slug_create_playlist, feedback_create_playlist) = if let Some(CollectionFormFeedback::CreatePlaylist { slug }) = &form_feedback {
        (
            form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        )
    } else {
        (
            form_field_pattern(&Field::valid(""), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            String::new()
        )
    };
    
    let (input_slug_create_subcollection, feedback_create_subcollection) = if let Some(CollectionFormFeedback::CreateSubcollection { slug }) = &form_feedback {
        (
            form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        )
    } else {
        (
            form_field_pattern(&Field::valid(""), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            String::new()
        )
    };

    let convert_to_playlist_form = if collection.playlists.is_empty() && collection.subcollections.is_empty() {
        let t_convert_to_playlist = language.translations.convert_to_playlist;
        formatdoc!(r#"
            <p>
                This collection contains no playlists or subcollections and can therefore
                seamlessly be converted to a playlist.
            </p>

            <form action="/convert-to-playlist/{path}" method="post">
                <button>{t_convert_to_playlist}</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let convert_to_video_form = if
        collection.playlists.is_empty() &&
        collection.subcollections.is_empty() &&
        collection.videos.len() < 2 {
        let message = if collection.videos.is_empty() {
            "This collection contains nothing can therefore seamlessly be converted to a video."
        } else {
            "This collection contains nothing can therefore seamlessly be converted to a video."
        };

        let t_convert_to_video = language.translations.convert_to_video;
        formatdoc!(r#"
            <p>{message}</p>

            <form action="/convert-to-video/{path}" method="post">
                <button>{t_convert_to_video}</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let input_unlisted_conditionally_hidden = match collection.path.as_str() {
        "" => if collection.unlisted { format!(r#"<input name="unlisted" type="hidden" value="checked">"#)} else { String::new() },
        _ => input_unlisted
    };

    // TODO: Placeholder-only indication of which is the label vs. url field is possibly
    //       not accessible. Check and address.
    let t_add = language.translations.add;
    let t_label = language.translations.label;
    let t_url = language.translations.url;
    let create_link_form = formatdoc!(r#"
        <form action="/add-collection-link/{path}" method="post">
            <div class="button_group">
                <input name="label" placeholder="{t_label}">
                <input name="url" placeholder="{t_url}">
                <button>{t_add}</button>
            </div>
        </form>
    "#);
    let links_form = if collection.links.is_empty() {
        create_link_form
    } else {
        let links_rendered = collection.links
            .iter()
            .map(|link| {
                let label = &link.label;
                let url = &link.url;

                let t_delete = language.translations.delete;
                let t_really_delete = language.translations.really_delete;
                let delete_icon = icons::failure(t_delete);
                let delete_form = formatdoc!(r#"
                    <form action="/delete-collection-link/{path}" method="post" onsubmit="return confirm('{t_really_delete}');" style="display: inline;">
                        <input name="label" type="hidden" value="{label}">
                        <input name="url" type="hidden" value="{url}">
                        <button class="delete_button">{delete_icon}</button>
                    </form>
                "#);

                formatdoc!(r#"
                    <div class="button_group">
                        <input value="{label}">
                        <input value="{url}">
                        {delete_form}
                    </div>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");
        formatdoc!(r#"
            {links_rendered}
            {create_link_form}
        "#)
    };

    let t_collection = language.translations.collection;
    let t_create_playlist = language.translations.create_playlist;
    let t_create_subcollection = language.translations.create_subcollection;
    let t_danger_zone = language.translations.danger_zone;
    let t_delete_collection = language.translations.delete_collection;
    let t_deleting_the_collection_text = language.translations.deleting_the_collection_text;
    let t_links = language.translations.links;
    let t_new_playlist = language.translations.new_playlist;
    let t_new_subcollection = language.translations.new_subcollection;
    let t_new_video = language.translations.new_video;
    let t_really_delete = language.translations.really_delete;
    let t_update_collection = language.translations.update_collection;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2>{t_collection}</h2>

            <form action="/collection/{path}" method="post">
                {input_title}
                {input_description}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_collection}</button> {feedback_update}
                </div>
            </form>
        </div>

        <div class="form_group">
            <h2>{t_links}</h2>
            {links_form}
        </div>

        <div class="form_group">
            <h2>{t_new_subcollection}</h2>

            <form action="/create-subcollection/{path}" method="post">
                {input_slug_create_subcollection}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_create_subcollection}</button> {feedback_create_subcollection}
                </div>
            </form>
        </div>

        <div class="form_group">
            <h2>{t_new_playlist}</h2>

            <form action="/create-playlist/{path}" method="post">
                {input_slug_create_playlist}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_create_playlist}</button> {feedback_create_playlist}
                </div>
            </form>
        </div>

        <div class="form_group">
            <h2>{t_new_video}</h2>
            {video_upload_button}
        </div>

        {site_edit}

        <div class="form_group">
            <h2>{t_danger_zone}</h2>

            {slug_form}

            {convert_to_playlist_form}

            {convert_to_video_form}

            <p>
                {t_deleting_the_collection_text}
            </p>

            <form action="/delete-collection/{path}" method="post">
                <button onclick="if (!confirm('{t_really_delete}')) {{ return false; }}">{t_delete_collection}</button>
            </form>
        </div>
    "#);

    let site = context.get_site();
    let head_title = collection.title.as_deref().unwrap_or_else(|| collection.slug());
    let html = layout(context, &language, Some(path), &site, &body, head_title);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn edit(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_collection(&path.path) {
        Some(collection) => edit_page(&collection, &context, None, &language),
        None => not_found(&context, &language, &path.path)
    }
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();

    let new_path = Path::new(&path.path)
        .with_file_name(slug_trimmed)
        .into_os_string()
        .into_string()
        .unwrap();

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    } else if context.site_dir.join(&new_path).exists() {
        Some("This permalink is already used by another collection, playlist or video.".to_string())
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |collection: &mut Collection| {
            fs::rename(
                context.site_dir.join(&collection.path),
                context.site_dir.join(&new_path)
            ).unwrap();

            collection.path = new_path;
        };

        let collection = match context.update_collection(mutation, &path.path) {
            Ok(collection) => collection,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback), &language))
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let description_trimmed = form.description.trim();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |collection: &mut Collection| {
            collection.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            collection.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            collection.unlisted = unlisted;
        };

        let collection = match context.update_collection(mutation, &path.path) {
            Ok(collection) => collection,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        collection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::Update {
            description: description_trimmed,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback), &language))
    }
}
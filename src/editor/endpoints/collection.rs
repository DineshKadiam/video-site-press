use std::fs;
use std::path::Path;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Either, Form, Path as ActixPath, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    Collection,
    Context,
    ContextPath,
    Field,
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
pub struct SlugForm {
    slug: String
}

#[derive(Deserialize)]
pub struct UpdateForm {
    description: String,
    title: String,
    unlisted: Option<String>
}

pub async fn create_playlist(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let slug_trimmed = form.slug.trim();

    let playlist_path = format!("{}/{slug_trimmed}", &path.path);

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
            return Either::Right(not_found(&context, &format!("Collection {} not found.", &path.path)));
        }

        playlist.write_manifest(&context.site_dir);

        let redirect_url = format!("/playlist/{}", playlist.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        let form_feedback = CollectionFormFeedback::CreatePlaylist {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback)))
    }
}

pub async fn create_subcollection(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let slug_trimmed = form.slug.trim();

    let subcollection_path = format!("{}/{slug_trimmed}", &path.path);

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
            return Either::Right(not_found(&context, &format!("Collection {} not found.", &path.path)));
        }

        subcollection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", subcollection.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        let form_feedback = CollectionFormFeedback::CreateSubcollection {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback)))
    }
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_collection(&path.path) {
        return Either::Right(not_found(&context, &format!("Collection {} not found.", &path.path)))
    };

    let redirect_url = if &path.path == "" {
        "/".to_string()
    } else {
        Path::new("/collection").join(&path.path).parent().unwrap().to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub fn edit_page(
    collection: &Collection,
    context: &Context,
    form_feedback: Option<CollectionFormFeedback>,
) -> HttpResponse {
    let path = &collection.path;

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

    let (
        feedback_update,
        input_description,
        input_title,
        input_unlisted,
    ) = if let Some(CollectionFormFeedback::Update { description, title, unlisted }) = &form_feedback {
        (
            r#"<span class="feedback">Not Saved</span>"#,
            form_textarea("description", "Description", "Description", Some(description)),
            form_field(title, "title", "Title", "Title"),
            form_checkbox(*unlisted, "unlisted", "Unlisted")
        )
    } else {
        (
            "",
            form_textarea("description", "Description", "Description", collection.description.as_deref()),
            form_field(&Field::valid(collection.title.as_deref().unwrap_or("")), "title", "Title", "Title"),
            form_checkbox(collection.unlisted, "unlisted", "Unlisted")
        )
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let (input, feedback) = if let Some(CollectionFormFeedback::Move { slug }) = &form_feedback {
            (
                form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                r#"<span class="feedback">Not Saved</span>"#
            )
        } else {
            (
                form_field_pattern(&Field::valid(collection.slug()), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                ""
            )
        };

        formatdoc!(r#"
            <p>
                Be aware that changing the slug/permalink of a collection changes
                the url under which the collection is publicly available on the web,
                or in other words, if there are links pointing to your collection
                right now, they will not work anymore when you change the slug.
            </p>

            <form action="/move-collection/{path}" method="post">
                {input}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>Update Permalink</button> {feedback}
                </div>
            </form>
        "#)
    };

    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, "Upload New Video");

    let site_form_feedback = match &form_feedback {
        Some(CollectionFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, path);

    let (input_slug_create_playlist, feedback_create_playlist) = if let Some(CollectionFormFeedback::CreatePlaylist { slug }) = &form_feedback {
        (
            form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            r#"<span class="feedback">Not Saved</span>"#
        )
    } else {
        (
            form_field_pattern(&Field::valid(""), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            ""
        )
    };
    
    let (input_slug_create_subcollection, feedback_create_subcollection) = if let Some(CollectionFormFeedback::CreateSubcollection { slug }) = &form_feedback {
        (
            form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            r#"<span class="feedback">Not Saved</span>"#
        )
    } else {
        (
            form_field_pattern(&Field::valid(""), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
            ""
        )
    };

    let convert_form = if collection.playlists.is_empty() && collection.subcollections.is_empty() {
        formatdoc!(r#"
            <p>
                This collection contains no playlists or subcollections and can therefore
                seamlessly be converted to a playlist.
            </p>

            <form action="/convert/playlist" method="post">
                <button>Convert collection to playlist</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let body = formatdoc!(r#"
        {errors}

        <h2>New Video</h2>

        {video_upload_button}

        <h2>New Playlist</h2>

        <form action="/create-playlist/{path}" method="post">
            {input_slug_create_playlist}

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Create Playlist</button> {feedback_create_playlist}
            </div>
        </form>

        <h2>New Subcollection</h2>

        <form action="/create-subcollection/{path}" method="post">
            {input_slug_create_subcollection}

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Create Subcollection</button> {feedback_create_subcollection}
            </div>
        </form>

        <h2>Collection</h2>

        <form action="/collection/{path}" method="post">
            {input_title}
            {input_description}
            {input_unlisted}

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Update Collection</button> {feedback_update}
            </div>
        </form>

        {site_edit}

        <h2>Danger Zone</h2>

        {slug_form}

        {convert_form}

        <p>
            Deleting the collection removes the collection manifest, all videos,
            playlists and subcollections. It does not remove the site manifest and
            the site itself, so the site can afterwards be reinitialized again as
            a collection, playlist or video.
        </p>

        <form action="/delete-collection/{path}" method="post">
            <button onclick="if (!confirm('Really delete?')) {{ return false; }}">Delete Collection</button>
        </form>
    "#);

    let site = context.get_site();
    let head_title = collection.title.as_deref().unwrap_or_else(|| collection.slug());
    let html = layout(context, path, &site, &body, head_title);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn edit(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> HttpResponse {
    match context.get_collection(&path.path) {
        Some(collection) => edit_page(&collection, &context, None),
        None => not_found(&context, &format!("Collection {} not found.", &path.path))
    }
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<SlugForm>,
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
            Err(()) => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        let form_feedback = CollectionFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback)))
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
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
            Err(()) => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        collection.write_manifest(&context);

        let redirect_url = format!("/collection/{}", &collection.path);

        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(
                not_found(&context, &format!("Collection {} not found.", &path.path))
            )
        };

        let form_feedback = CollectionFormFeedback::Update {
            description: description_trimmed,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(edit_page(&collection, &context, Some(form_feedback)))
    }
}
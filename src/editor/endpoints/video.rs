use std::fs;
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
use slug::slugify;

use crate::{
    Collection,
    Context,
    ContextPath,
    Field,
    FileMeta,
    GetAnyResult,
    HyperDir,
    Job,
    JobKind,
    Playlist,
    PosterFile,
    Video,
    VideoFile
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
use crate::util::{precise_duration, human_size, nanoid, validate_slug};

pub enum VideoFormFeedback<'a> {
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
pub struct VideoMoveForm {
    slug: String
}

#[derive(Deserialize)]
pub struct VideoUpdateForm {
    description: String,
    title: String,
    unlisted: Option<String>
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_video(&path.path) {
        return Either::Right(not_found(&context, &format!("Video {} not found.", &path.path)))
    };

    let redirect_url = if &path.path == "" {
        "/".to_string()
    } else {
        // TODO: The video might have been in a collection OR playlist, so we need to dynamically
        // find out what it was and use the correct url. (or some other solution e.g. where we
        // don't need to spell out collection/playlist/video in the url, somewhat like /any/.*)
        Path::new("/collection").join(&path.path).parent().unwrap().to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<VideoFormFeedback>,
    video: &Video
) -> HttpResponse {
    let errors = if video.errors.is_empty() {
        String::new()
    } else {
        video.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let path = &video.path;

    let versions = if video.versions.is_empty() {
        String::new()
    } else {
        let versions = video.versions
            .iter()
            .map(|version| {
                let video_meta = match &version.video_meta {
                    Some(video_meta) => format!(
                        "{}×{} {} {}",
                        video_meta.width,
                        video_meta.height,
                        precise_duration(video_meta.duration),
                        human_size(version.file_meta.size)
                    ),
                    None => String::from("Metadata pending")
                };

                let file_name = &version.file_name;

                formatdoc!(r#"
                    <li>
                        {file_name} ({video_meta})
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        formatdoc!(r#"
            <h2>Video/Audio Versions</h2>

            <ul>
                {versions}
            </ul>
        "#)
    };

    // let in_playlists = {
    //     let playlists = context.collection.get_playlists_for_video(&video);

    //     if playlists.is_empty() {
    //         String::new()
    //     } else {
    //         let list = playlists
    //             .into_iter()
    //             .map(|playlist| {
    //                 let id = &playlist.id;
    //                 let title = &playlist.title;
                    
    //                 format!(r#"<a href="/playlist/{id}">{title}</a>"#)
    //             })
    //             .collect::<Vec<String>>()
    //             .join(", ");

    //         formatdoc!("
    //             <br><br>
    //             <div>
    //                 In playlists: {list}
    //             </div>"
    //         )
    //     }
    // };

    let poster = video.poster
        .as_ref()
        .map(|poster| {
            let file_name = &poster.file_name;
            let src = if path == "" {
                format!("/poster/{file_name}")
            } else {
                format!("/poster/{path}/{file_name}")
            };
            format!(r#"<img class="thumbnail" src="{src}">"#)
        })
        .unwrap_or(String::new());

    let poster_upload_button = upload_widget(&format!("/video-poster/{path}"), "Upload Poster");

    let (
        feedback_update,
        input_description,
        input_title,
        input_unlisted,
    ) = if let Some(VideoFormFeedback::Update { description, title, unlisted }) = &form_feedback {
        (
            r#"<span class="feedback">Not Saved</span>"#,
            form_textarea("description", "Description", "Description", Some(description)),
            form_field(title, "title", "Title", "Title"),
            form_checkbox(*unlisted, "unlisted", "Unlisted")
        )
    } else {
        (
            "",
            form_textarea("description", "Description", "Description", video.description.as_deref()),
            form_field(&Field::valid(video.title.as_deref().unwrap_or("")), "title", "Title", "Title"),
            form_checkbox(video.unlisted, "unlisted", "Unlisted")
        )
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let (input, feedback) = if let Some(VideoFormFeedback::Move { slug }) = &form_feedback {
            (
                // TODO: Double-check this pattern (and elsewhere checks from start to end ($^?))
                form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                r#"<span class="feedback">Not Saved</span>"#
            )
        } else {
            (
                form_field_pattern(&Field::valid(video.slug()), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                ""
            )
        };

        formatdoc!(r#"
            <p>
                Be aware that changing the slug/permalink of a video changes
                the url under which the video is publicly available on the web,
                or in other words, if there are links pointing to your video
                right now, they will not work anymore when you change the slug.
            </p>

            <form action="/move-video/{path}" method="post">
                {input}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>Update Permalink</button> {feedback}
                </div>
            </form>
        "#)
    };

    let site_form_feedback = match &form_feedback {
        Some(VideoFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(&context, site_form_feedback, &video.path);

    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, "Upload New Version");

    let body = formatdoc!(r#"
        {errors}

        <h2>Video</h2>

        <form action="/video/{path}" method="post">
            {input_title}
            {input_description}
            {input_unlisted}

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Update Video</button> {feedback_update}
            </div>
        </form>
        
        {versions}

        {video_upload_button}

        <h2>Poster Image</h2>

        {poster}
        {poster_upload_button}
        <form action="/video-random-poster/{path}" enctype="multipart/form-data" method="post">
            <button>Randomly pick another poster frame (unimplemented)</button>
        </form>

        {site_edit}

        <h2>Danger Zone</h2>

        {slug_form}

        <p>
            Deleting the video removes the video manifest, poster image,
            and all versions of the video. If the video lies at the
            site root it does not remove the site manifest and the site itself,
            so the site can afterwards be reinitialized again as a collection,
            video or video.
        </p>

        <form action="/delete-video/{path}" method="post">
            <button onclick="if (!confirm('Really delete?')) {{ return false; }}">Delete Video</button>
        </form>
    "#);

    let site = &context.site.read().unwrap();
    let head_title = video.title.as_deref().unwrap_or(video.slug());
    let html = layout(context, path, &site, &body, head_title);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn edit(context: Data<Arc<Context>>, path: ActixPath<ContextPath>) -> HttpResponse {
    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => return not_found(&context, &format!("Video {} not found.", &path.path))
    };

    edit_page(&context, None, &video)
}

// TODO: We currently go through the video endpoint for poster for all posters
//       (also playlist e.g.), this is unclean, make generic or change otherwise
// TODO: Highly insecure b/c path arbitrary etc. you know
pub async fn poster(context: Data<Arc<Context>>, path: ActixPath<ContextPath>) -> HttpResponse {
    let filepath = context.site_dir.join(&path.path);
    let content = web::block(move || fs::read(filepath))
        .await
        .unwrap()
        .unwrap();

    HttpResponse::Ok()
        // TODO: Dynamic, correct content type
        .content_type(ContentType::jpeg())
        .body(content)
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<VideoMoveForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let slug_trimmed = form.slug.trim();

    let slug_error = if let Err(message) = validate_slug(slug_trimmed) {
        Some(message)
    // } else if let Some(owner) = context.collection.slug_collision(slug_trimmed, &path.id) {
    //     Some(format!("Already used by another playlist or video: {owner}"))
    } else {
        None
    };

    if slug_error.is_none() {
        let mutation = |video: &mut Video| {
            // TODO: Here and elsewhere: Changing the slug works very differently now (!) b/c it's encoded in the filesystem path
            // video.slug = slug_trimmed.to_owned();
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        video.persist_in_site(&context);

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        let form_feedback = VideoFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &video)
        )
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<VideoUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let description_trimmed = form.description.trim();
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();

    // TODO: We're keeping this around for now because we might need to re-introduce some actual validation? (description/title length e.g.?)
    if true {
        let mutation = |video: &mut Video| {
            video.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video.unlisted = unlisted;
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        video.persist_in_site(&context);

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        let form_feedback = VideoFormFeedback::Update {
            description: description_trimmed,
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &video)
        )
    }
}

pub async fn upload_poster(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            let filename_lowercase = filename.to_lowercase();
            // TODO: DRY up with other places where this is done (+expand formats)
            if !(filename_lowercase.ends_with(".jpg") ||
                filename_lowercase.ends_with(".jpeg") ||
                filename_lowercase.ends_with(".png") ||
                filename_lowercase.ends_with(".webp")) {

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
            let mut f = web::block(|| std::fs::File::create(storage_path_moving)).await??;

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
            }

            let file_meta = FileMeta::new(&storage_path);
            let poster_file = PosterFile::new(None, file_meta, storage_filename);

            let mutation = |video: &mut Video| {
                video.poster = Some(poster_file);
            };

            let video = match context.update_video(mutation, &path.path) {
                Ok(video) => video,
                Err(()) => {
                    fs::remove_file(&storage_path).unwrap();
                    let http_response = not_found(&context, &format!("Video {} not found.", &path.path));
                    return Ok(Either::Right(http_response));
                }
            };

            video.persist_in_site(&context);

            let job = Job::new(JobKind::Poster, Arc::clone(&video));
            context.job_queue.lock().unwrap().push(job);

            let redirect_url = format!("/video/{}", path.path);
            return Ok(Either::Left(Redirect::to(redirect_url).see_other()));
        }
    }

    unreachable!()
}

// TODO: Implementation wise this is very messy, refactoring candidate
pub async fn upload_video(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>,
    mut payload: Multipart
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let container = context.get_any(&path.path);

    if let GetAnyResult::None = container {
        let http_response = not_found(&context, &format!("No collection, playlist or video found at path {}", &path.path));
        return Ok(Either::Right(http_response));
    }

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            // TODO: Support all the formats plz, also audio (?!)
            if filename.ends_with(".mp4") {
                let original_filename = filename.to_string();

                let storage_path = match &container {
                    GetAnyResult::Collection(collection) => {
                        let mut candidate_dir_name = slugify(Path::new(&original_filename).with_extension("").to_string_lossy());
                        while context.site_dir.join(&collection.path).join(&candidate_dir_name).exists() {
                            candidate_dir_name.push_str("-duplicate");
                        }
                        let candidate_dir = context.site_dir.join(&collection.path).join(&candidate_dir_name);
                        fs::create_dir_all(&candidate_dir).unwrap();
                        candidate_dir.join(original_filename)
                    }
                    GetAnyResult::Playlist(playlist) => {
                        let mut candidate_dir_name = slugify(Path::new(&original_filename).with_extension("").to_string_lossy());
                        while context.site_dir.join(&playlist.path).join(&candidate_dir_name).exists() {
                            candidate_dir_name.push_str("-duplicate");
                        }
                        let candidate_dir = context.site_dir.join(&playlist.path).join(&candidate_dir_name);
                        fs::create_dir_all(&candidate_dir).unwrap();
                        candidate_dir.join(original_filename)
                    }
                    GetAnyResult::Video(video) => {
                        let mut candidate_filename = original_filename.clone();
                        while context.site_dir.join(&video.path).join(&candidate_filename).exists() {
                            candidate_filename = Path::new(&candidate_filename).with_file_name(format!("{}.mp4", nanoid())).to_string_lossy().to_string();
                        }
                        context.site_dir.join(&video.path).join(&candidate_filename)
                    }
                    GetAnyResult::None => unreachable!()
                };

                let storage_path_moving = storage_path.clone();
                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| std::fs::File::create(storage_path_moving)).await??;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                // TODO: path.path must be verified, and video actually appended in to the site tree
                // let video = Video::new_upload(&original_filename, &storage_filename, &path.path);

                match &container {
                    GetAnyResult::Collection(_) => {
                        let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                        let video = Video::read_dir(&context, &hyper_dir);

                        let video_moving = Arc::clone(&video);
                        let mutation = |collection: &mut Collection| {
                            collection.videos.push(video_moving);
                        };

                        if context.update_collection(mutation, &path.path).is_err() {
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            return Ok(Either::Right(not_found(&context, &format!("Collection {} not found.", &path.path))));
                        }

                        let metadata_job = Job::new(JobKind::Metadata, Arc::clone(&video));
                        let poster_job = Job::new(JobKind::Poster, Arc::clone(&video));

                        {
                            let mut job_queue = context.job_queue.lock().unwrap();
                            job_queue.push(metadata_job);
                            job_queue.push(poster_job);
                        }

                        // TODO: Do we really need to persist anything here? Video is implicitly video anyway through the video file ...
                        // collection.persist_in_site(&context.site_dir);

                        let redirect_url = format!("/video/{}", &video.path);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                    }
                    GetAnyResult::Playlist(_) => {
                        let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                        let video = Video::read_dir(&context, &hyper_dir);

                        let video_moving = Arc::clone(&video);
                        let mutation = |playlist: &mut Playlist| {
                            playlist.videos_physical.push(video_moving);
                        };

                        if context.update_playlist(mutation, &path.path).is_err() {
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            return Ok(Either::Right(not_found(&context, &format!("Playlist {} not found.", &path.path))));
                        }

                        // TODO: Do we really need to persist anything here? Video is implicitly video anyway through the video file ...
                        // playlist.persist_in_site(&context.site_dir);

                        let metadata_job = Job::new(JobKind::Metadata, Arc::clone(&video));
                        let poster_job = Job::new(JobKind::Poster, Arc::clone(&video));

                        {
                            let mut job_queue = context.job_queue.lock().unwrap();
                            job_queue.push(metadata_job);
                            job_queue.push(poster_job);
                        }

                        let redirect_url = format!("/video/{}", &video.path);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                    }
                    GetAnyResult::Video(_) => {
                        let file_meta = FileMeta::new(&storage_path);
                        let file_name = storage_path.file_name().unwrap().to_string_lossy().to_string();

                        let mutation = |video: &mut Video| {
                            video.versions.push(VideoFile::new(file_meta, file_name, None));
                        };

                        let video = match context.update_video(mutation, &path.path) {
                            Ok(video) => video,
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            Err(()) => return Ok(Either::Right(not_found(&context, &format!("Video {} not found.", &path.path))))
                        };

                        let metadata_job = Job::new(JobKind::Metadata, Arc::clone(&video));

                        {
                            let mut job_queue = context.job_queue.lock().unwrap();
                            job_queue.push(metadata_job);

                            if video.poster.is_none() {
                                let poster_job = Job::new(JobKind::Poster, Arc::clone(&video));
                                job_queue.push(poster_job);
                            }
                        }

                        let redirect_url = format!("/video/{}", &video.path);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                    }
                    GetAnyResult::None => unreachable!()
                }
            }
        }
    }

    let http_response = not_found(&context, "No actionable payload found in request");
    return Ok(Either::Right(http_response));
}
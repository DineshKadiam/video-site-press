use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{self, Data, Form, Path as ActixPath, Redirect};
use chrono::NaiveDate;
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
    VideoFile,
    VideoFormat,
    VideoParent
};
use crate::editor::endpoints;
use crate::editor::widgets::{
    form_checkbox,
    form_field_date,
    form_field,
    form_field_pattern,
    form_textarea,
    layout,
    not_found,
    upload_widget
};
use crate::util::{
    precise_duration,
    human_size,
    nanoid,
    supported_image_extension,
    supported_video_extension,
    validate_slug
};

#[derive(Deserialize)]
pub struct DownsizeForm {
    width: String
}

pub enum VideoFormFeedback<'a> {
    Move {
        slug: Field
    },
    Site(endpoints::site::SiteFormFeedback),
    Update {
        description: &'a str,
        release_date: Field,
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
    release_date: String,
    title: String,
    unlisted: Option<String>
}

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let video_parent = match context.delete_video(&path.path) {
        Ok(video_parent) => video_parent,
        Err(()) => return Either::Right(not_found(&context, &format!("Video {} not found.", &path.path)))
    };

    let redirect_url = match video_parent {
        Some(VideoParent::Collection(collection)) => {
            Path::new("/collection").join(&collection.path).to_str().unwrap().to_string()
        }
        Some(VideoParent::Playlist(playlist)) => {
            Path::new("/playlist").join(&playlist.path).to_str().unwrap().to_string()
        }
        None => "/".to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

pub async fn downsize(
    context: Data<Arc<Context>>,
    form: Form<DownsizeForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => return Either::Right(not_found(&context, &format!("Video {} not found.", &path.path)))
    };

    // TODO: Build programmatically in code instead of parsing from format string
    let video_format_string = format!("video:h264:mp4:speed:1.0:{}", form.width);
    let video_format = VideoFormat::parse(&video_format_string).unwrap();

    let job = Job::new(JobKind::Format(video_format), Arc::clone(&video));
    context.job_queue.lock().unwrap().push(job);

    let redirect_url = format!("/video/{}", &path.path);
    Either::Left(Redirect::to(redirect_url).see_other())
}

fn downsize_form(video: &Video) -> String {
    let widest_width = if let Some(version) = video.highest_resolution_version() {
        version.video_meta.as_ref().unwrap().width
    } else {
        return String::new();
    };

    let mut options = Vec::new();
    let candidate_widths = [240, 480, 720, 1280, 1920];
    for width in candidate_widths {
        if width < widest_width {
            let option = formatdoc!(r#"
                <option value="{width}">{width} pixels wide</option>
            "#);
            options.push(option);
        }
    }

    if options.is_empty() {
        return String::new();
    }

    let options = options.join("\n");
    let path = &video.path;

    formatdoc!(r#"
        <form action="/downsize/{path}" method="post">
            <select name="width">
                {options}
            </select>

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Request downsized version</button>
            </div>
        </form>
    "#)
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
                    Some(video_meta) => {
                        let duration = precise_duration(video_meta.duration);
                        let height = video_meta.height;
                        let size = human_size(version.file_meta.size);
                        let width = video_meta.width;

                        let browser_support = match video_meta.browser_support.analysis() {
                            Some(Ok(message)) => format!("[Very likely playable in browsers: {message}]"),
                            Some(Err(message)) => format!("[Unfit for browser playback: {message}]"),
                            None => "[Might be incompatible with and not play in people's browsers]".to_string(),
                        };

                        format!("{width}×{height} {duration} {size} {browser_support}")
                    }
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

        let downsize_form_rendered = downsize_form(video);

        formatdoc!(r#"
            <h2>Video/Audio Versions</h2>

            <ul>
                {versions}
            </ul>

            {downsize_form_rendered}
        "#)
    };

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

    let poster_upload_button = upload_widget(&format!("/upload-video-poster/{path}"), "Upload Poster");

    let (
        feedback_update,
        input_description,
        input_release_date,
        input_title,
        input_unlisted,
    ) = if let Some(VideoFormFeedback::Update { description, title, release_date, unlisted }) = &form_feedback {
        (
            r#"<span class="feedback">Not Saved</span>"#,
            form_textarea("description", "Description", "Description", Some(description)),
            form_field_date(release_date, "release_date", "Release Date"),
            form_field(title, "title", "Title", "Title"),
            form_checkbox(*unlisted, "unlisted", "Unlisted")
        )
    } else {
        let release_date = video.release_date
            .as_ref()
            .map(|date| date.to_string())
            .unwrap_or(String::new());

        (
            "",
            form_textarea("description", "Description", "Description", video.description.as_deref()),
            form_field_date(&Field::valid(release_date), "release_date", "Release Date"),
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
    let site_edit = endpoints::site::edit(context, site_form_feedback, &video.path);

    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, "Upload New Version");

    let body = formatdoc!(r#"
        {errors}

        <h2>Video</h2>

        <form action="/video/{path}" method="post">
            {input_title}
            {input_description}
            {input_release_date}
            {input_unlisted}

            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Update Video</button> {feedback_update}
            </div>
        </form>
        
        {versions}

        <br><br>

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

    let site = context.get_site();
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
    let filepath_moving = filepath.clone();
    let content = web::block(move || fs::read(filepath_moving))
        .await
        .unwrap()
        .unwrap();

    match filepath.extension() {
        Some(extension) => {
            let content_type = if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpg") {
                "image/jpeg"
            } else if extension.eq_ignore_ascii_case("png") {
                "image/png"
            } else if extension.eq_ignore_ascii_case("webp") {
                "image/webp"
            } else if extension.eq_ignore_ascii_case("webp") {
                "image/gif"
            } else {
                let body = format!("Poster file extension {} can not be requested.", extension.to_string_lossy());
                return HttpResponse::NotImplemented().body(body);
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .body(content)
        }
        None => {
            HttpResponse::NotImplemented()
                .body("Files without extension can not be requested as a poster.")
        }
    }
}

pub async fn r#move(
    context: Data<Arc<Context>>,
    form: Form<VideoMoveForm>,
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
        let mutation = |video: &mut Video| {
            fs::rename(
                context.site_dir.join(&video.path),
                context.site_dir.join(&new_path)
            ).unwrap();

            video.path = new_path;
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

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
    let release_date = if form.release_date.is_empty() {
        None
    } else {
        Some(
            NaiveDate::parse_from_str(&form.release_date, "%Y-%m-%d")
                .map_err(|err| err.to_string())
        )
    };
    let title_trimmed = form.title.trim();
    let unlisted = form.unlisted.is_some();



    if release_date.as_ref().is_some_and(|release_date| release_date.is_err()) {
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        let form_feedback = VideoFormFeedback::Update {
            description: description_trimmed,
            release_date: Field::validated(&form.release_date, release_date.unwrap().err()),
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &video)
        )
    } else {
        let mutation = |video: &mut Video| {
            video.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video.release_date = release_date.and_then(|release_date| release_date.ok());
            video.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video.unlisted = unlisted;
            video.write_manifest(&context);
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(
                not_found(&context, &format!("Video {} not found.", &path.path))
            )
        };

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
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
            if !supported_image_extension(filename) {
                // TODO: Implement without panicking - return something instead
                panic!("Unsupported file extension in {filename}");
            }

            let original_filename = filename.to_string();

            let storage_filename = if !context.site_dir.join(&path.path).join(&original_filename).exists() {
                original_filename
            } else {
                format!("{}_{original_filename}", nanoid())
            };

            let storage_path = context.site_dir.join(&path.path).join(&storage_filename);
            let storage_path_moving = storage_path.clone();

            // File::create is blocking operation, use threadpool
            let mut f = web::block(|| File::create(storage_path_moving)).await??;

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
            }

            let file_meta = FileMeta::new(&storage_path);
            let poster_file = PosterFile::new(None, file_meta, storage_filename);

            let mutation = |video: &mut Video| {
                video.poster = Some(poster_file);
                video.write_manifest(&context);
            };

            let video = match context.update_video(mutation, &path.path) {
                Ok(video) => video,
                Err(()) => {
                    fs::remove_file(&storage_path).unwrap();
                    let http_response = not_found(&context, &format!("Video {} not found.", &path.path));
                    return Ok(Either::Right(http_response));
                }
            };

            let job = Job::new(JobKind::FastTasks, Arc::clone(&video));
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
            // TODO: Support audio formats as well?
            //       Or should this be a separate endpoint/feature?
            if supported_video_extension(filename) {
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
                let mut f = web::block(|| File::create(storage_path_moving)).await??;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

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

                        let job = Job::new(JobKind::FastTasks, Arc::clone(&video));
                        context.job_queue.lock().unwrap().push(job);

                        let redirect_url = format!("/video/{}", &video.path);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                    }
                    GetAnyResult::Playlist(_) => {
                        let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                        let video = Video::read_dir(&context, &hyper_dir);

                        let video_moving = Arc::clone(&video);
                        let mutation = |playlist: &mut Playlist| {
                            playlist.videos.push(video_moving);
                        };

                        if context.update_playlist(mutation, &path.path).is_err() {
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            return Ok(Either::Right(not_found(&context, &format!("Playlist {} not found.", &path.path))));
                        }

                        let job = Job::new(JobKind::FastTasks, Arc::clone(&video));
                        context.job_queue.lock().unwrap().push(job);

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

                        let job = Job::new(JobKind::FastTasks, Arc::clone(&video));
                        context.job_queue.lock().unwrap().push(job);

                        let redirect_url = format!("/video/{}", &video.path);
                        return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                    }
                    GetAnyResult::None => unreachable!()
                }
            }
        }
    }

    let http_response = not_found(&context, "No actionable payload found in request");
    Ok(Either::Right(http_response))
}
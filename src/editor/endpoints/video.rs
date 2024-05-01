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
    JobKind,
    Playlist,
    PosterFile,
    Video,
    VideoFile,
    VideoFormat,
    VideoParent
};
use crate::browser_support;
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
pub struct DeleteVersionForm {
    file_name: String
}

#[derive(Deserialize)]
pub struct DownsizeForm {
    approach: String,
    codec: String,
    height: String,
    quality: String
}

#[derive(Deserialize)]
pub struct PosterUpdateForm {
    description: String
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
    },
    UploadPoster {
        error: String
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

pub async fn delete_poster(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video: &mut Video| {
        if let Some(poster) = &video.poster {
            let poster_path = context.site_dir.join(&video.path).join(&poster.file_name);
            video.poster = None;
            video.write_manifest(&context);
            let _ = fs::remove_file(poster_path);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => Either::Right(
            not_found(&context, &format!("Video {} not found.", &path.path))
        )
    }
}

pub async fn delete_version(
    context: Data<Arc<Context>>,
    form: Form<DeleteVersionForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    // TODO: Minor issue if at all, but there is no feedback if the version is not found

    let mutation = |video_mut: &mut Video| {
        if let Some(version) = video_mut.versions
            .iter()
            .position(|version| version.file_name == form.file_name)
            .map(|index| video_mut.versions.swap_remove(index)) {
            let version_path = context.site_dir.join(&path.path).join(&version.file_name);
            video_mut.write_manifest(&context);
            let _ = fs::remove_file(&version_path);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => Either::Right(
            not_found(&context, &format!("Video {} not found.", &path.path))
        )
    }
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

    // TODO: Safety validation of these maybe
    let approach = &form.approach;
    let codec = &form.codec;
    let height = &form.height;
    let quality = &form.quality;

    // TODO: Build programmatically in code instead of parsing from format string
    let video_format_string = format!("video:{codec}:mp4:{approach}:{quality}:{height}");
    let video_format = VideoFormat::parse(&video_format_string).unwrap();

    let job_kind = JobKind::VideoFormat { format: video_format, video: Arc::clone(&video) }; 
    context.push_job_queue(job_kind);

    let redirect_url = format!("/video/{}", &path.path);
    Either::Left(Redirect::to(redirect_url).see_other())
}

fn additional_formats_form(context: &Context, video: &Video) -> String {
    let (aspect_ratio, tallest_height) = if let Some(version) = video.highest_resolution_version() {
        let video_meta = version.video_meta.as_ref().unwrap();
        (video_meta.width as f32 / video_meta.height as f32, video_meta.height)
    } else {
        return String::new();
    };

    let mut height_options = Vec::new();
    let candidate_heights = [144, 240, 360, 480, 720, 1080];
    for height in candidate_heights {
        if height <= tallest_height {
            let width = (height as f32 * aspect_ratio) as usize;

            let selected = if (tallest_height < 720 && height == 480) || height == 720 {
                "selected"
            } else {
                ""
            };

            let option = formatdoc!(r#"
                <option {selected} value="{height}">{width}×{height}</option>
            "#);
            height_options.push(option);
        }
    }

    if height_options.is_empty() {
        return String::new();
    }

    let height_options = height_options.join("\n");
    let path = &video.path;

    let t_best = &context.editor_translations.best;
    let t_codec = &context.editor_translations.codec;
    let t_encoding_approach = &context.editor_translations.encoding_approach;
    let t_maximum_efficiency_best_compression_through_slowest_encoding = &context.editor_translations.maximum_efficiency_best_compression_through_slowest_encoding;
    let t_maximum_speed_fastest_encoding_through_worst_compression = &context.editor_translations.maximum_speed_fastest_encoding_through_worst_compression;
    let t_quality = &context.editor_translations.quality;
    let t_recommended_codec = &context.editor_translations.recommended_codec;
    let t_render_additional_format = &context.editor_translations.render_additional_format;
    let t_render_format = &context.editor_translations.render_format;
    let t_resolution = &context.editor_translations.resolution;
    let t_worst = &context.editor_translations.worst;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_render_additional_format}</h2>
            <form action="/downsize/{path}" method="post">
                <div class="form_field">
                    <label for="resolution">{t_resolution}</label>
                    <select id="resolution" name="height">
                        {height_options}
                    </select>
                </div>
                <div class="form_field">
                    <label for="quality">{t_quality}</label>
                    <div style="align-items: center; column-gap: .3rem; display: flex;">
                        <span>{t_worst}</span>
                        <div id="quality_wrapper" style="flex: 1; position: relative;">
                            <span style="left: calc(0.95 * 80%); position: absolute; top: -1rem;">80%</span>
                            <input
                                aria-valuenow="80%"
                                aria-valuetext="80%"
                                id="quality"
                                max="1.0"
                                min="0.0"
                                name="quality"
                                step=".01"
                                style="width: 100%;"
                                type="range"
                                value="0.8">
                        </div>
                        <span>{t_best}</span>
                    </div>
                </div>
                <div class="form_field">
                    <label for="codec">{t_codec}</label>
                    <select id="codec" name="codec">
                        <option value="h264">H264 ({t_recommended_codec})</option>
                        <option value="hevc">HEVC</option>
                        <option value="vp9">VP9</option>
                    </select>
                </div>
                <div class="form_field">
                    <label for="approach">{t_encoding_approach}</label>
                    <select id="approach" name="approach">
                        <option value="efficiency">{t_maximum_efficiency_best_compression_through_slowest_encoding}</option>
                        <option value="speed">{t_maximum_speed_fastest_encoding_through_worst_compression}</option>
                    </select>
                </div>
                <button>{t_render_format}</button>
            </form>
        </div>
    "#)
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<VideoFormFeedback>,
    video: &Video
) -> HttpResponse {
    let t_not_saved = &context.editor_translations.not_saved;

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
                let (browser_support, video_meta) = match &version.video_meta {
                    Some(video_meta) => {
                        let duration = precise_duration(video_meta.duration);
                        let height = video_meta.height;
                        let size = human_size(version.file_meta.size);
                        let width = video_meta.width;

                        let (browser_support, reasons) = match browser_support::analysis(video_meta) {
                            Some(Ok(message)) => (&context.editor_translations.good_browser_support, message),
                            Some(Err(message)) => (&context.editor_translations.not_support_by_browsers, message),
                            None => (&context.editor_translations.browser_support_unclear, String::from("Hyper 8 could not determine whether this video's format is generally compatible with browsers or not"))
                        };

                        (
                            format!(r#"<div class="browser_support">{browser_support} <span class="reasons" title="{reasons}">🛈</span></div>"#),
                            format!("{width}×{height} {duration} {size}")
                        )
                    }
                    None => (
                        String::new(),
                        String::from("Metadata pending")
                    )
                };

                let file_name = &version.file_name;

                let t_really_delete = &context.editor_translations.really_delete;
                let delete_form = formatdoc!(r#"
                    <form action="/delete-version/{path}" method="post" onsubmit="return confirm('{t_really_delete}');" style="display: inline;">
                        <input name="file_name" type="hidden" value="{file_name}">
                        <button style="display: inline;">×</button>
                    </form>
                "#);

                formatdoc!(r#"
                    <li>
                        <div>
                            <span class="file_name">{file_name}</span> <span class="metadata">{video_meta}</span> {delete_form}
                        </div>
                        {browser_support}
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_available_formats = &context.editor_translations.available_formats;
        formatdoc!(r#"
            <h2>{t_available_formats}</h2>

            <ul class="versions">
                {versions}
            </ul>
        "#)
    };

    let t_upload_poster = &context.editor_translations.upload_poster;
    let poster_upload_button = upload_widget(&format!("/upload-poster/{path}"), t_upload_poster);
    let poster_upload_button_error = if let Some(VideoFormFeedback::UploadPoster { error }) = &form_feedback {
        format!(r#"<span class="error">{error}</span>"#)
    } else {
        String::new()
    };
    let poster_form = match &video.poster {
        Some(poster) => {
            let alt = if let Some(description) = &poster.description {
                format!(r#"alt="{description}" "#)
            } else {
                String::new()
            };

            // TODO: Could/should also use computed assets (also elsewhere, videos etc.)
            let file_name = &poster.file_name;
            let src = if path == "" {
                format!("/poster/{file_name}")
            } else {
                format!("/poster/{path}/{file_name}")
            };

            let t_poster_description = &context.editor_translations.poster_description;
            let description = form_textarea("description", t_poster_description, poster.description.as_deref());

            let t_delete_poster = &context.editor_translations.delete_poster;
            let t_random_poster = &context.editor_translations.random_poster;
            let t_update_description = &context.editor_translations.update_description;
            format!(r#"
                <img {alt}class="thumbnail" src="{src}">
                <div class="button_group">
                    {poster_upload_button}
                    <form action="/delete-poster/{path}" method="post">
                        <div style="align-items: center; column-gap: .5rem; display: flex;">
                            <button>{t_delete_poster}</button>
                        </div>
                    </form>
                    <form action="/random-poster/{path}" method="post">
                        <button>{t_random_poster}</button>
                    </form>
                </div>
                {poster_upload_button_error}
                <form action="/update-poster/{path}" method="post">
                    {description}

                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button>{t_update_description}</button>
                    </div>
                </form>
            "#)
        }
        None => {
            formatdoc!(r#"
                {poster_upload_button}
                {poster_upload_button_error}
            "#)
        }
    };

    let t_description = &context.editor_translations.description;
    let t_release_date = &context.editor_translations.release_date;
    let t_title = &context.editor_translations.title;
    let t_unlisted = &context.editor_translations.unlisted;
    let (
        feedback_update,
        input_description,
        input_release_date,
        input_title,
        input_unlisted,
    ) = if let Some(VideoFormFeedback::Update { description, title, release_date, unlisted }) = &form_feedback {
        (
            format!(r#"<span class="feedback">{t_not_saved}</span>"#),
            form_textarea("description", t_description, Some(description)),
            form_field_date(release_date, "release_date", t_release_date),
            form_field(title, "title", t_title),
            form_checkbox(*unlisted, "unlisted", t_unlisted)
        )
    } else {
        let release_date = video.release_date
            .as_ref()
            .map(|date| date.to_string())
            .unwrap_or(String::new());

        (
            String::new(),
            form_textarea("description", t_description, video.description.as_deref()),
            form_field_date(&Field::valid(release_date), "release_date", t_release_date),
            form_field(&Field::valid(video.title.as_deref().unwrap_or("")), "title", t_title),
            form_checkbox(video.unlisted, "unlisted", t_unlisted)
        )
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let (input, feedback) = if let Some(VideoFormFeedback::Move { slug }) = &form_feedback {
            (
                // TODO: Double-check this pattern (and elsewhere checks from start to end ($^?))
                form_field_pattern(slug, "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                format!(r#"<span class="feedback">{t_not_saved}</span>"#)
            )
        } else {
            (
                form_field_pattern(&Field::valid(video.slug()), "slug", "Permalink", "Permalink", r#"[A-Za-z0-9\-]+"#),
                String::new()
            )
        };

        let t_a_permalink_defines_under_which_url_text = &context.editor_translations.a_permalink_defines_under_which_url_text;
        let t_update_permalink = &context.editor_translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_defines_under_which_url_text}
            </p>

            <form action="/move-video/{path}" method="post">
                {input}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let site_form_feedback = match &form_feedback {
        Some(VideoFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, &video.path);

    let t_upload_additional_format = &context.editor_translations.upload_additional_format;
    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, t_upload_additional_format);

    // TODO: Implement for non-root/non-site videos
    let t_convert_to_video_in_collection = &context.editor_translations.convert_to_video_in_collection;
    let t_convert_to_video_in_playlist = &context.editor_translations.convert_to_video_in_playlist;
    let convert_form = formatdoc!(r#"
        <p>
            A video can seamlessly be converted to a video in a collection.
        </p>

        <form action="/convert-to-collection/{path}" method="post">
            <button>{t_convert_to_video_in_collection}</button>
        </form>

        <p>
            A video can seamlessly be converted to a video in a playlist.
        </p>

        <form action="/convert-to-playlist/{path}" method="post">
            <button>{t_convert_to_video_in_playlist}</button>
        </form>
    "#);

    let input_unlisted_conditionally_hidden = match video.path.as_str() {
        "" => if video.unlisted { format!(r#"<input name="unlisted" type="hidden" value="checked">"#)} else { String::new() },
        _ => input_unlisted
    };

    let render_additional_formats = additional_formats_form(context, video);

    let t_danger_zone = &context.editor_translations.danger_zone;
    let t_delete_video = &context.editor_translations.delete_video;
    let t_poster_image = &context.editor_translations.poster_image;
    let t_really_delete = &context.editor_translations.really_delete;
    let t_video = &context.editor_translations.video;
    let t_update_video = &context.editor_translations.update_video;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2>{t_video}</h2>

            <form action="/video/{path}" method="post">
                {input_title}
                {input_description}
                {input_release_date}
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_video}</button> {feedback_update}
                </div>
            </form>
        </div>
        
        <div class="form_group">
            {versions}
            {video_upload_button}
        </div>

        {render_additional_formats}

        <div class="form_group">
            <h2>{t_poster_image}</h2>
            {poster_form}
        </div>

        {site_edit}

        <div class="form_group">
            <h2>{t_danger_zone}</h2>

            {slug_form}

            {convert_form}

            <p>
                Deleting the video removes the video manifest, poster image,
                and all versions of the video. If the video lies at the
                site root it does not remove the site manifest and the site itself,
                so the site can afterwards be reinitialized again as a collection,
                playlist or video.
            </p>

            <form action="/delete-video/{path}" method="post">
                <button onclick="if (!confirm('{t_really_delete}')) {{ return false; }}">{t_delete_video}</button>
            </form>
        </div>
    "#);

    let site = context.get_site();
    let head_title = video.title.as_deref().unwrap_or(video.slug());
    let html = layout(context, Some(path), &site, &body, head_title);

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

pub async fn random_poster(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    match context.get_video(&path.path) {
        Some(video) => {
            let site = context.get_site();
            let job_kind = JobKind::VideoRandomPoster {
                poster_aspect: site.poster_aspect.numeric(),
                video: Arc::clone(&video)
            };
            context.push_job_queue(job_kind);

            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => Either::Right(
            not_found(&context, &format!("Video {} not found.", &path.path))
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
        let mutation = |video_mut: &mut Video| {
            video_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video_mut.release_date = release_date.and_then(|release_date| release_date.ok());
            video_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video_mut.unlisted = unlisted;
            video_mut.write_manifest(&context);
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

pub async fn update_poster(
    context: Data<Arc<Context>>,
    form: Form<PosterUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video: &mut Video| {
        if let Some(poster) = &mut video.poster {
            poster.description = Some(form.description.clone());
        }
        video.write_manifest(&context);
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => Either::Right(
            not_found(&context, &format!("Video {} not found.", &path.path))
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
            if !supported_image_extension(filename) {
                match context.get_video(&path.path) {
                    Some(video) => {
                        let error = format!("Unsupported file extension in {filename}");
                        let form_feedback = VideoFormFeedback::UploadPoster { error };
                        let http_response = edit_page(&context, Some(form_feedback), &video);
                        return Ok(Either::Right(http_response))
                    }
                    None => {
                        let http_response = not_found(&context, &format!("Video {} not found.", &path.path));
                        return Ok(Either::Right(http_response));
                    }
                }
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
            let poster_file = PosterFile::new(None, None, file_meta, storage_filename);

            let mutation = |video: &mut Video| {
                video.poster = Some(poster_file);
                video.write_manifest(&context);
            };

            let video = match context.update_video(mutation, &path.path) {
                Ok(video) => video,
                Err(()) => {
                    let _ = fs::remove_file(&storage_path);
                    let http_response = not_found(&context, &format!("Video {} not found.", &path.path));
                    return Ok(Either::Right(http_response));
                }
            };

            let site = context.get_site();
            let job_kind = JobKind::VideoFastTasks {
                poster_aspect: site.poster_aspect.numeric(),
                video: Arc::clone(&video)
            };
            context.push_job_queue(job_kind);

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

                let site = context.get_site();

                let video = match &container {
                    GetAnyResult::Collection(_) => {
                        let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                        let video = Video::read_dir(&context, &hyper_dir, &site);

                        let video_moving = Arc::clone(&video);
                        let mutation = |collection: &mut Collection| {
                            collection.videos.push(video_moving);
                        };

                        if context.update_collection(mutation, &path.path).is_err() {
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            return Ok(Either::Right(not_found(&context, &format!("Collection {} not found.", &path.path))));
                        }

                        video
                    }
                    GetAnyResult::Playlist(_) => {
                        let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                        let video = Video::read_dir(&context, &hyper_dir, &site);

                        let video_moving = Arc::clone(&video);
                        let mutation = |playlist: &mut Playlist| {
                            playlist.videos.push(video_moving);
                        };

                        if context.update_playlist(mutation, &path.path).is_err() {
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            return Ok(Either::Right(not_found(&context, &format!("Playlist {} not found.", &path.path))));
                        }

                        video
                    }
                    GetAnyResult::Video(_) => {
                        let file_meta = FileMeta::new(&storage_path);
                        let file_name = storage_path.file_name().unwrap().to_string_lossy().to_string();

                        let mutation = |video: &mut Video| {
                            video.versions.push(VideoFile::new(file_meta, file_name, None));
                        };

                        match context.update_video(mutation, &path.path) {
                            Ok(video) => video,
                            // TODO: Can only happen in weird time of check vs. time of use edge cases
                            Err(()) => return Ok(Either::Right(not_found(&context, &format!("Video {} not found.", &path.path))))
                        }
                    }
                    GetAnyResult::None => unreachable!()
                };

                let job_kind = JobKind::VideoFastTasks {
                    poster_aspect: site.poster_aspect.numeric(),
                    video: Arc::clone(&video)
                };
                context.push_job_queue(job_kind);

                let redirect_url = format!("/video/{}", &video.path);
                return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
            }
        }
    }

    let http_response = not_found(&context, "No actionable payload found in request");
    Ok(Either::Right(http_response))
}
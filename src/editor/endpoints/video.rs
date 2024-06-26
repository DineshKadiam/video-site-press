// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    HyperUrl,
    JobKind,
    Language,
    Link,
    Playlist,
    PosterFile,
    Translations,
    Video,
    VideoFile,
    VideoFormat,
    VideoParent
};
use crate::browser_support;
use crate::editor::{Checkbox, Input, Select, Textarea};
use crate::editor::endpoints;
use crate::editor::widgets::{
    add_link_form,
    error_page,
    layout,
    links,
    not_found,
    preview_button,
    upload_widget
};
use crate::icons;
use crate::util::{
    precise_duration,
    human_size,
    nanoid,
    supported_image_extension,
    supported_video_extension,
    validate_slug
};

#[derive(Deserialize)]
pub struct AddLinkForm {
    label: String,
    url: String,
    verification: String
}

#[derive(Deserialize)]
pub struct DeleteLinkForm {
    label: String,
    url: String
}

#[derive(Deserialize)]
pub struct DeleteVersionForm {
    file_name: String
}

#[derive(Deserialize)]
pub struct DownsizeForm {
    approach: String,
    codec: String,
    frame_rate: String,
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
        sort_number: Field,
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
    sort_number: String,
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

    let mutation = |video_mut: &mut Video| {
        let link = Link::new(hidden, label, rel_me, url);
        video_mut.links.push(link);
        video_mut.write_manifest(&context);
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
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
    let video_parent = match context.delete_video(&path.path) {
        Ok(video_parent) => video_parent,
        Err(()) => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
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

pub async fn delete_link(
    context: Data<Arc<Context>>,
    form: Form<DeleteLinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if video_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url.representation == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url.representation == form.url
                }
            )
            .map(|index| video_mut.links.remove(index))
            .is_some() {
            video_mut.write_manifest(&context);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn delete_poster(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |video_mut: &mut Video| {
        if let Some(poster) = &video_mut.poster {
            let poster_path = context.site_dir.join(&video_mut.path).join(&poster.file_name);
            video_mut.poster = None;
            video_mut.write_manifest(&context);
            let _ = fs::remove_file(poster_path);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
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
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn downsize(
    context: Data<Arc<Context>>,
    form: Form<DownsizeForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
    };

    let frame_rate = if form.frame_rate.is_empty() {
        None
    } else {
        Some(form.frame_rate.as_str())
    };

    let video_format = VideoFormat::new_video(
        &form.codec,
        "mp4",
        &form.approach,
        frame_rate,
        &form.height,
        &form.quality,
    ).unwrap();

    let job_kind = JobKind::VideoFormat { format: video_format, video: Arc::clone(&video) }; 
    context.push_job_queue(job_kind);

    let redirect_url = format!("/video/{}", &path.path);
    Either::Left(Redirect::to(redirect_url).see_other())
}

fn additional_formats_form(translations: &Translations, video: &Video) -> String {
    let (aspect_ratio, highest_frame_rate, tallest_height) = if let Some(version) = video.highest_quality_version() {
        let video_meta = version.video_meta.as_ref().unwrap();
        (
            video_meta.width as f32 / video_meta.height as f32,
            video_meta.video_frame_rate,
            video_meta.height
        )
    } else {
        return String::new();
    };

    let t_original = translations.original;

    let mut select_resolution = Select::new("height", translations.resolution);
    select_resolution.id("resolution");

    let mut no_height_options = true;
    let candidate_heights = [1080, 720, 480, 360, 240, 144];
    let padding_len = tallest_height.to_string().len();
    for height in candidate_heights {
        if height <= tallest_height {
            let width = (height as f32 * aspect_ratio) as usize;

            let original = if height == tallest_height {
                format!("({t_original})")
            } else {
                String::new()
            };

            let padding = "&nbsp;".repeat(2 * (padding_len - height.to_string().len()));
            let label = format!(r#"<wbr>{padding}{height}p – {width}×{height} {original}"#);

            select_resolution.option(height.to_string(), label);

            no_height_options = false;
        }
    }

    if no_height_options {
        return String::new();
    }

    let mut select_frame_rate = Select::new("frame_rate", translations.frame_rate);

    let candidate_frame_rates = [60.0, 48.0, 30.0, 25.0, 24.0, 12.0];
    let known_frame_rate = candidate_frame_rates.iter().any(|frame_rate| *frame_rate == highest_frame_rate);
    if !known_frame_rate {
        let label = format!(r#"{highest_frame_rate} fps ({t_original})"#);
        select_frame_rate.option("", label);
        select_frame_rate.selected("");
    }
    for frame_rate in candidate_frame_rates {
        if frame_rate <= highest_frame_rate {
            if known_frame_rate && frame_rate == highest_frame_rate {
                let label = format!("{frame_rate} fps ({t_original})");
                select_frame_rate.option("", label);
                select_frame_rate.selected("");
            } else {
                let label = format!("{frame_rate} fps");
                select_frame_rate.option(frame_rate.to_string(), label);
            }
        }
    }

    let mut select_codec = Select::new("codec", translations.codec);

    select_codec.option("h264", translations.h264_supported_in_all_browsers_recommended);
    select_codec.option("hevc", translations.hevc_h265_missing_browser_support_not_recommended);
    select_codec.option("vp9", translations.vp9_missing_browser_support_not_recommended);

    let mut select_approach = Select::new("approach", translations.encoding_approach_label);

    select_approach.option("compression", translations.highest_compression_recommended_but_takes_a_long_time);
    select_approach.option("average", translations.average_compression_and_speed_reasonable_for_temporarily_used_media);
    select_approach.option("speed", translations.highest_speed_only_use_if_you_need_results_as_soon_as_possible);

    select_approach.selected("compression");

    let path = &video.path;

    let t_best = translations.best;
    let t_quality = translations.quality;
    let t_render_additional_format = translations.render_additional_format;
    let t_render_format = translations.render_format;
    let t_worst = translations.worst;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_render_additional_format}</h2>
            <form action="/downsize/{path}" method="post">
                <div class="form_split">
                    {select_resolution}
                    {select_frame_rate}
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
                {select_codec}
                {select_approach}
                <button class="action">{t_render_format}</button>
            </form>
        </div>
    "#)
}

pub fn edit_page(
    context: &Context,
    form_feedback: Option<VideoFormFeedback>,
    language: &Language,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &language.translations;
    let t_not_saved = translations.not_saved;

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
        let versions = video.versions_desc_by_quality();

        let mut duration_range = None;

        if versions.len() > 1 {
            for version in &versions {
                if let Some(video_meta) = &version.video_meta {
                    match &mut duration_range {
                        Some((shortest_version, longest_version)) => {
                            if video_meta.duration > *longest_version {
                                *longest_version = video_meta.duration;
                            }

                            if video_meta.duration > *shortest_version {
                                *shortest_version = video_meta.duration;
                            }
                        }
                        None => {
                            duration_range = Some((video_meta.duration, video_meta.duration));
                        }
                    }
                }
            }
        }

        let duration_info = if let Some((shortest_version, longest_version)) = duration_range {
            if longest_version - shortest_version < 1.0 {
                let duration = precise_duration(longest_version);

                let t_all_formats_are_the_same_length = translations.all_formats_are_the_same_length;
                let t_ok = translations.ok;
                let success_icon = icons::success(t_ok);
                format!(r#"<div class="duration_info">{success_icon} {t_all_formats_are_the_same_length} ({duration})</div>"#)
            } else {
                let longest = precise_duration(longest_version);
                let shortest = precise_duration(shortest_version);

                let t_not_ok = translations.not_ok;
                let t_length_diverges_between_different_formats = translations.length_diverges_between_different_formats;
                let failure_icon = icons::failure(t_not_ok);
                format!(r#"<div class="duration_info">{failure_icon} {t_length_diverges_between_different_formats} ({shortest} - {longest})</div>"#)
            }
        } else {
            String::new()
        };

        let versions_rendered = versions
            .iter()
            .map(|version| {
                let (browser_support, label, video_meta) = match &version.video_meta {
                    Some(video_meta) => {
                        let duration = precise_duration(video_meta.duration);
                        let frame_rate = video_meta.video_frame_rate;
                        let height = video_meta.height;
                        let size = human_size(version.file_meta.size);
                        let width = video_meta.width;

                        let (browser_support, reasons) = match browser_support::analysis(&version.file_name, &language, video_meta) {
                            Some(Ok(message)) => (translations.good_browser_support, message),
                            Some(Err(message)) => (translations.not_support_by_browsers, message),
                            None => (translations.browser_support_unclear, String::from("Hyper 8 could not determine whether this video's format is generally compatible with browsers or not"))
                        };

                        (
                            format!(r#"<small class="browser_support">{browser_support} <span class="reasons" title="{reasons}">🛈</span></small>"#),
                            format!("{height}p {frame_rate}fps {size}"),
                            format!("{width}×{height} {duration}")
                        )
                    }
                    None => (
                        String::new(),
                        String::new(),
                        String::from("Metadata pending")
                    )
                };

                let file_name = &version.file_name;

                let t_delete = translations.delete;
                let t_really_delete = translations.really_delete;
                let delete_icon = icons::failure(t_delete);
                let delete_form = formatdoc!(r#"
                    <form
                    action="/delete-version/{path}"
                    data-confirm
                    data-confirm-label="{t_delete}"
                    data-confirm-text="{t_really_delete}"
                    method="post"
                    style="display: inline;">
                        <input name="file_name" type="hidden" value="{file_name}">
                        <button class="danger delete_button">{delete_icon}</button>
                    </form>
                "#);

                formatdoc!(r#"
                    <li>
                        <div>
                            <div class="basics">{label} {delete_form}</div>
                            <small>{video_meta} {file_name}</small>
                        </div>
                        {browser_support}
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_available_formats = translations.available_formats;
        formatdoc!(r#"
            <h2>{t_available_formats}</h2>

            {duration_info}

            <ul class="versions">
                {versions_rendered}
            </ul>
        "#)
    };

    let t_upload_poster = translations.upload_poster;
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

            let mut description = Textarea::image_description("description", translations.poster_description);

            description
                .id("poster_description")
                .value_option(&poster.description);

            let t_delete_poster = translations.delete_poster;
            let t_random_poster = translations.random_poster;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;
            format!(r#"
                <img {alt}class="thumbnail" src="{src}">
                <div class="button_group">
                    {poster_upload_button}
                    <form action="/random-poster/{path}" method="post">
                        <button class="action">{t_random_poster}</button>
                    </form>
                    <form action="/delete-poster/{path}" data-confirm data-confirm-label="{t_delete_poster}" data-confirm-text="{t_really_delete}" method="post">
                        <div style="align-items: center; column-gap: .5rem; display: flex;">
                            <button class="danger">{t_delete_poster}</button>
                        </div>
                    </form>
                </div>
                {poster_upload_button_error}
                <form action="/update-poster/{path}" data-warn-discard method="post">
                    {description}

                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button class="action">{t_update_description}</button>
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

    let mut input_release_date = Input::date("release_date", translations.release_date);
    let mut input_sort_number = Input::number("sort_number", translations.sort_number);
    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut textarea_description = Textarea::markdown_subset(language, "description", translations.description);

    textarea_description.id("video_description");

    let feedback_update = if let Some(VideoFormFeedback::Update { description, release_date, sort_number, title, unlisted }) = &form_feedback {
        input_release_date.validated_value(release_date);
        input_sort_number.validated_value(sort_number);
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        textarea_description.value(description);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let release_date = video.release_date.as_ref().map(|date| date.to_string());
        let sort_number = video.sort_number.as_ref().map(|number| number.to_string());

        input_release_date.value_option(&release_date);
        input_sort_number.value_option(&sort_number);
        input_title.value_option(&video.title);
        input_unlisted.checked(video.unlisted);
        textarea_description.value_option(&video.description);

        String::new()
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let mut input_permalink = Input::permalink("slug", translations.permalink);

        input_permalink.required();

        let feedback = if let Some(VideoFormFeedback::Move { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(video.slug());
            String::new()
        };

        let t_a_permalink_defines_under_which_url_text = translations.a_permalink_defines_under_which_url_text;
        let t_update_permalink = translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_defines_under_which_url_text}
            </p>

            <form action="/move-video/{path}" data-warn-discard method="post">
                {input_permalink}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="danger">{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let site_form_feedback = match &form_feedback {
        Some(VideoFormFeedback::Site(site_form_feedback)) => Some(site_form_feedback),
        _ => None
    };
    let site_edit = endpoints::site::edit(context, site_form_feedback, &video.path, translations);

    let t_upload_new_version = translations.upload_new_version;
    let upload_url = format!("/upload-video/{path}");
    let video_upload_button = upload_widget(&upload_url, t_upload_new_version);

    let t_this_video_can_be_seamlessly_converted_to_a_video_in_a_collection = translations.this_video_can_be_seamlessly_converted_to_a_video_in_a_collection;
    let t_this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist = translations.this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist;
    let t_move_into_new_collection = translations.move_into_new_collection;
    let t_move_into_new_playlist = translations.move_into_new_playlist;
    let convert_form = formatdoc!(r#"
        <p>
            {t_this_video_can_be_seamlessly_converted_to_a_video_in_a_collection}
        </p>
        <form action="/convert-to-collection/{path}" method="post">
            <button class="danger">{t_move_into_new_collection}</button>
        </form>
        <p>
            {t_this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist}
        </p>
        <form action="/convert-to-playlist/{path}" method="post">
            <button class="danger">{t_move_into_new_playlist}</button>
        </form>
    "#);

    let input_unlisted_conditionally_hidden = match video.path.as_str() {
        "" => if video.unlisted { format!(r#"<input name="unlisted" type="hidden" value="checked">"#)} else { String::new() },
        _ => input_unlisted.to_string()
    };

    let render_additional_formats = additional_formats_form(&translations, video);

    let create_link_form = add_link_form(&format!("/add-video-link/{path}"), translations);
    let links_form = links(&video.links, &format!("/delete-video-link/{path}"), translations);

    let r_preview_button = preview_button(path, translations);

    let t_danger_zone = translations.danger_zone;
    let t_delete = translations.delete;
    let t_delete_video = translations.delete_video;
    let t_deleting_the_video_text = translations.deleting_the_video_text;
    let t_poster_image = translations.poster_image;
    let t_really_delete = translations.really_delete;
    let t_video = translations.video;
    let t_update_video = translations.update_video;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                <span>{t_video}</span>
                {r_preview_button}
            </h2>

            <form action="/video/{path}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                <div class="form_split divide_3_1">
                    {input_release_date}
                    {input_sort_number}
                </div>
                {input_unlisted_conditionally_hidden}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_video}</button> {feedback_update}
                </div>
            </form>
        </div>

        {links_form}

        {create_link_form}

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
                {t_deleting_the_video_text}
            </p>

            <form
                action="/delete-video/{path}"
                data-confirm
                data-confirm-label="{t_delete}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_video}</button>
            </form>
        </div>
    "#);

    let head_title = video.title.as_deref().unwrap_or(video.slug());
    let html = layout(context, language, Some(path), &site, &body, head_title);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub async fn edit(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> HttpResponse {
    let language = context.get_editor_language();

    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => return not_found(&context, &language, &path.path)
    };

    edit_page(&context, None, &language, &video)
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
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
        };

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    } else {
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = VideoFormFeedback::Move {
            slug: Field::validated(slug_trimmed, slug_error)
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &video)
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
        None => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<VideoUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let description_trimmed = form.description.trim();

    let release_date_trimmed = form.release_date.trim();
    let release_date = if release_date_trimmed.is_empty() {
        None
    } else {
        Some(
            NaiveDate::parse_from_str(&release_date_trimmed, "%Y-%m-%d")
                .map_err(|err| err.to_string())
        )
    };

    let sort_number_trimmed = form.sort_number.trim();
    let sort_number = if sort_number_trimmed.is_empty() {
        None
    } else {
        Some(
            sort_number_trimmed.parse::<i32>().map_err(|err| err.to_string())
        )
    };

    let title_trimmed = form.title.trim();

    let unlisted = form.unlisted.is_some();

    if release_date.as_ref().is_some_and(|release_date| release_date.is_err()) ||
        sort_number.as_ref().is_some_and(|sort_number| sort_number.is_err()) {
        let video = match context.get_video(&path.path) {
            Some(video) => video,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = VideoFormFeedback::Update {
            description: description_trimmed,
            release_date: Field::validated(release_date_trimmed, release_date.and_then(|release_date| release_date.err())),
            sort_number: Field::validated(sort_number_trimmed, sort_number.and_then(|sort_number| sort_number.err())),
            title: Field::valid(title_trimmed),
            unlisted
        };

        Either::Right(
            edit_page(&context, Some(form_feedback), &language, &video)
        )
    } else {
        let mutation = |video_mut: &mut Video| {
            video_mut.description = if description_trimmed.is_empty() { None } else { Some(description_trimmed.to_owned()) };
            video_mut.release_date = release_date.and_then(|release_date| release_date.ok());
            video_mut.sort_number = sort_number.and_then(|sort_number| sort_number.ok());
            video_mut.title = if title_trimmed.is_empty() { None } else { Some(title_trimmed.to_owned()) };
            video_mut.unlisted = unlisted;
            video_mut.write_manifest(&context);
        };

        let video = match context.update_video(mutation, &path.path) {
            Ok(video) => video,
            Err(()) => return Either::Right(not_found(&context, &language, &path.path))
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
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}

pub async fn upload_poster(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            if !supported_image_extension(filename) {
                match context.get_video(&path.path) {
                    Some(video) => {
                        let error = format!("Unsupported file extension in {filename}");
                        let form_feedback = VideoFormFeedback::UploadPoster { error };
                        let http_response = edit_page(&context, Some(form_feedback), &language, &video);
                        return Ok(Either::Right(http_response))
                    }
                    None => {
                        let http_response = not_found(&context, &language, &path.path);
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
                    let http_response = not_found(&context, &language, &path.path);
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
    let language = context.get_editor_language();

    let container = context.get_any(&path.path);

    if let GetAnyResult::None = container {
        let http_response = not_found(&context, &language, &path.path);
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
                            return Ok(Either::Right(not_found(&context, &language, &path.path)));
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
                            return Ok(Either::Right(not_found(&context, &language, &path.path)));
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
                            Err(()) => return Ok(Either::Right(not_found(&context, &language, &path.path)))
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

    unimplemented!()
}
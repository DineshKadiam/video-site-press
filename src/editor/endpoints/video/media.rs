// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use crate::{
    Context,
    Language,
    SitePath,
    Translations,
    Video
};
use crate::browser_support;
use crate::editor::{Location, Select, Textarea};
use crate::editor::routes;
use crate::editor::widgets::{layout, not_found, upload_widget};
use crate::icons;
use crate::util::{precise_duration, human_size};

pub struct UploadPosterFormFeedback {
    pub error: String
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

    select_frame_rate.option("", translations.automatic_original_frame_rate_or_encoder_preference);
    select_frame_rate.selected("");

    let candidate_frame_rates = [60.0, 48.0, 30.0, 25.0, 24.0, 12.0];
    // If the original frame rate is one we don't normally offer, offer it separately here
    if !candidate_frame_rates.iter().any(|frame_rate| *frame_rate == highest_frame_rate) {
        let label = format!(r#"{highest_frame_rate} fps ({t_original})"#);
        select_frame_rate.option(highest_frame_rate.to_string(), label);
    }
    for frame_rate in candidate_frame_rates {
        if frame_rate <= highest_frame_rate {
            let label = if frame_rate == highest_frame_rate {
                format!("{frame_rate} fps ({t_original})")
            } else {
                format!("{frame_rate} fps")
            };
            select_frame_rate.option(frame_rate.to_string(), label);
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

    let downsize_action = routes::video_downsize(video);

    let t_advanced_settings = translations.advanced_settings;
    let t_best = translations.best;
    let t_quality = translations.quality;
    let t_render_additional_format = translations.render_additional_format;
    let t_render_format = translations.render_format;
    let t_worst = translations.worst;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_render_additional_format}</h2>
            <form action="{downsize_action}" method="post">
                <div class="button_group">
                    {select_resolution}
                    <div class="form_field">
                        <label>&nbsp;</label>
                        <button class="action">{t_render_format}</button>
                    </div>
                </div>
                <details>
                    <summary>{t_advanced_settings}</summary>
                    <div class="form_field">
                        {select_frame_rate}
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
                </details>
            </form>
        </div>
    "#)
}

pub async fn media(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_video(&site_path) {
        Some(video) => {
            media_page(
                &context,
                None,
                &language,
                &video
            )
        }
        None => return not_found(&context, &language, &site_path)
    }
}

pub fn media_page(
    context: &Context,
    form_feedback: Option<UploadPosterFormFeedback>,
    language: &Language,
    video: &Video
) -> HttpResponse {

    let site = context.get_site();
    let translations = &language.translations;

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
                String::new()
            } else {
                let longest = precise_duration(longest_version);
                let shortest = precise_duration(shortest_version);

                let t_not_ok = translations.not_ok;
                let t_length_diverges_between_different_formats = translations.length_diverges_between_different_formats;
                let failure_icon = icons::failure(t_not_ok);
                format!(r#"<div class="error">{failure_icon} {t_length_diverges_between_different_formats} ({shortest} - {longest})</div>"#)
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

                        let (browser_support, reasons) = match browser_support::analysis(&version.file_name, language, video_meta) {
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
                let delete_version_action = routes::video_delete_version(video);
                let delete_form = formatdoc!(r#"
                    <form
                    action="{delete_version_action}"
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
    let poster_upload_button = upload_widget(&routes::video_upload_poster(video), t_upload_poster);
    let poster_upload_button_error = if let Some(UploadPosterFormFeedback { error }) = &form_feedback {
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
            let src = routes::poster(file_name, &video.site_path);

            let mut description = Textarea::image_description("description", translations.poster_description);

            description
                .id("poster_description")
                .value_option(&poster.description);

            let t_delete_poster = translations.delete_poster;
            let t_random_poster = translations.random_poster;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;

            let delete_poster_action = routes::video_delete_poster(video);
            let random_poster_action = routes::video_random_poster(video);
            let update_poster_action = routes::video_update_poster(video);

            format!(r#"
                <img {alt}class="thumbnail" src="{src}">
                <div class="button_group">
                    {poster_upload_button}
                    <form action="{random_poster_action}" method="post">
                        <button class="action">{t_random_poster}</button>
                    </form>
                    <form action="{delete_poster_action}" data-confirm data-confirm-label="{t_delete_poster}" data-confirm-text="{t_really_delete}" method="post">
                        <div style="align-items: center; column-gap: .5rem; display: flex;">
                            <button class="danger">{t_delete_poster}</button>
                        </div>
                    </form>
                </div>
                {poster_upload_button_error}
                <form action="{update_poster_action}" data-warn-discard method="post">
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

    let t_upload_new_version = translations.upload_new_version;
    let upload_url = routes::upload_video(&video.site_path);
    let video_upload_button = upload_widget(&upload_url, t_upload_new_version);

    let render_additional_formats = additional_formats_form(translations, video);

    let t_poster_image = translations.poster_image;

    let body = formatdoc!(r#"
        <div class="form_group">
            {versions}
            {video_upload_button}
        </div>

        {render_additional_formats}

        <div class="form_group">
            <h2>{t_poster_image}</h2>
            {poster_form}
        </div>
    "#);

    let t_media = translations.media;
    let video_title = video.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_media} – {video_title}");
    let html = layout(
        &body,
        context,
        language,
        Location::media(&video.site_path),
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

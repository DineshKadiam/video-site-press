// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{
    Desc,
    SitePath,
    Video
};
use hyper8_core::{BrowserSupport, MediaUsage};
use hyper8_translations::Translations;
use hyper8_util::{precise_duration, human_size};

use crate::{
    Context,
    Location,
    Select,
    SessionState
};
use crate::routes;
use crate::widgets::{layout, not_found, upload_widget};

fn additional_formats_form(translations: &Translations, video: &Video) -> String {
    let video_meta = if let Some(video_file) = video.highest_quality_video_file() {
        match &video_file.video_meta {
            Some(video_meta) => video_meta,
            None => {
                let jobs_url = routes::JOBS;

                // TODO: Translate/improve wording/usability
                return formatdoc!(r#"
                    Video metadata has not yet been computed – please click
                    <a href="">refresh</a> in a few seconds to get re-encoding
                    options for this video.
                    <br><br>
                    <small>
                        If this persists, please have a look at the
                        <a href="{jobs_url}">background jobs page</a>
                        to check for potential errors during metadata
                        computation.
                    </small>
                "#);
            }
        }
    } else {
        return String::new();
    };

    let mut select_audio_format = Select::new("format", translations.codec);
    select_audio_format.option("mp3", translations.mp3_highest_compatibility);
    select_audio_format.option("opus", translations.opus_recommended_highest_efficiency);
    select_audio_format.selected("opus");

    let t_original = translations.original;

    let mut select_resolution = Select::new("height", translations.resolution);
    select_resolution.id("resolution");

    let height_padding_len = video_meta.height.to_string().len();

    let mut add_resolution_option = |height: usize, original: bool, width: usize| {
        let original = if original { format!(" ({t_original})") } else { String::new() };
        let padding = "&nbsp;".repeat(2 * (height_padding_len - height.to_string().len()));
        let label = format!(r#"<wbr>{padding}{height}p – {width}×{height}{original}"#);

        select_resolution.option(height.to_string(), label);
    };

    // Allow re-encoding at the original video dimensions (e.g. just to
    // increase compression or change framerate)
    add_resolution_option(video_meta.height, true, video_meta.width);

    // Allow resizing to smaller resolutions
    let aspect_ratio = video_meta.aspect_ratio();
    for height in [1080, 720, 480, 360, 240, 144] {
        if height < video_meta.height {
            let width = (height as f32 * aspect_ratio) as usize;
            add_resolution_option(height, false, width);
        }
    }

    let mut select_frame_rate = Select::new("frame_rate", translations.frame_rate);

    select_frame_rate.option("", translations.automatic_original_frame_rate_or_encoder_preference);
    select_frame_rate.selected("");

    let mut add_frame_rate_option = |frame_rate: f32, original: bool| {
        let original = if original { format!(" ({t_original})") } else { String::new() };
        let label = format!("{frame_rate} fps{original}");

        select_frame_rate.option(frame_rate.to_string(), label);
    };

    add_frame_rate_option(video_meta.video_frame_rate, true);

    for frame_rate in [60.0, 48.0, 30.0, 25.0, 24.0, 12.0] {
        if frame_rate <= video_meta.video_frame_rate {
            add_frame_rate_option(frame_rate, false);
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

    let encode_audio_action = routes::video_encode_audio(video);
    let encode_video_action = routes::video_encode_video(video);

    let t_advanced_settings = translations.advanced_settings;
    let t_best = translations.best;
    let t_encode_audio = translations.encode_audio;
    let t_quality = translations.quality;
    let t_encode_video = translations.encode_video;
    let t_render_format = translations.render_format;
    let t_worst = translations.worst;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_encode_video}</h2>
            <form action="{encode_video_action}" method="post">
                {select_resolution}
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
                <button class="action">{t_render_format}</button>
            </form>
        </div>
        <div class="form_group">
            <h2>{t_encode_audio}</h2>
            <form action="{encode_audio_action}" method="post">
                {select_audio_format}
                <button class="action">{t_render_format}</button>
            </form>
        </div>
    "#)
}

pub async fn media(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_video(&site_path) {
        Some(video) => {
            media_page(
                &context,
                &session_state,
                &video
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn media_page(
    context: &Context,
    session_state: &SessionState,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;

    let t_usage_restrictions = translations.usage_restrictions;

    let audio_files = if video.audio_files.is_empty() {
        String::new()
    } else {
        let mut duration_range = None;

        if video.audio_files.len() > 1 {
            for audio_file in &video.audio_files {
                if let Some(audio_meta) = &audio_file.audio_meta {
                    match &mut duration_range {
                        Some((shortest_audio_file, longest_audio_file)) => {
                            if audio_meta.duration > *longest_audio_file {
                                *longest_audio_file = audio_meta.duration;
                            }

                            if audio_meta.duration > *shortest_audio_file {
                                *shortest_audio_file = audio_meta.duration;
                            }
                        }
                        None => {
                            duration_range = Some((audio_meta.duration, audio_meta.duration));
                        }
                    }
                }
            }
        }

        let duration_info = if let Some((shortest_audio_file, longest_audio_file)) = duration_range {
            if longest_audio_file - shortest_audio_file < 1.0 {
                String::new()
            } else {
                let longest = precise_duration(longest_audio_file);
                let shortest = precise_duration(shortest_audio_file);

                let t_not_ok = translations.not_ok;
                let t_length_diverges_between_different_formats = translations.length_diverges_between_different_formats;
                let failure_icon = hyper8_icons::failure(t_not_ok);
                format!(r#"<div class="error">{failure_icon} {t_length_diverges_between_different_formats} ({shortest} - {longest})</div>"#)
            }
        } else {
            String::new()
        };

        let audio_files_rendered = video.audio_files
            .iter()
            .map(|audio_file| {
                let file_name = &audio_file.file_name;
                let extension = file_name.rsplit_once('.').unwrap().1;
                let size = human_size(audio_file.file_meta.size);

                let attachment_type;
                let duration;
                let r_browser_support;

                // TODO: See if we even need audio_meta to determine browser support here
                match &audio_file.audio_meta {
                    Some(audio_meta) => {
                        duration = precise_duration(audio_meta.duration);

                        let reasons;
                        let support;

                        match BrowserSupport::audio_analysis(
                            audio_meta,
                            &audio_file.file_name,
                            &translations
                        ) {
                            BrowserSupport::Supported(verdict) => {
                                attachment_type = "attached_success";
                                reasons = verdict.explanation;
                                support = format!("{} – {}", translations.good_browser_support, verdict.summary);
                            }
                            BrowserSupport::Unknown => {
                                attachment_type = "attached_warning";
                                reasons = translations.hyper_8_could_not_determine_whether_this_file_format_is_generally_compatible_with_browsers_or_not.to_string();
                                support = translations.browser_support_unclear.to_string();
                            }
                            BrowserSupport::Unsupported(verdict) => {
                                attachment_type = "attached_error";
                                reasons = verdict.explanation;
                                support = format!("{} – {}", translations.not_supported_by_browsers, verdict.summary);
                            }
                        }

                        r_browser_support = format!(
                            r#"{support} <span class="reasons" title="{reasons}">🛈</span>"#
                        );
                    }
                    None => {
                        attachment_type = "attached_info";
                        duration = String::new();
                        r_browser_support = translations.metadata_pending.to_string();
                    }
                }

                let t_delete = translations.delete;
                let t_really_delete = translations.really_delete;
                let delete_icon = hyper8_icons::failure(t_delete);
                let delete_audio_file_action = routes::video_delete_audio_file(video);
                let delete_form = formatdoc!(r#"
                    <form
                        action="{delete_audio_file_action}"
                        data-confirm
                        data-confirm-label="{t_delete}"
                        data-confirm-text="{t_really_delete}"
                        method="post">
                        <input name="file_name" type="hidden" value="{file_name}">
                        <button>{delete_icon}</button>
                    </form>
                "#);

                let usage_options = MediaUsage::variants(translations)
                    .iter()
                    .map(|(label, usage)| {
                        let key = usage.key();
                        let selected = if *usage == audio_file.usage { "selected" } else { "" };

                        format!(r#"<option {selected} value="{key}">{label}</option>"#)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                let update_usage_url = routes::video_update_audio_file_usage(video);

                formatdoc!(r#"
                    <li>
                        <div class="actionable_item attachment {attachment_type}">
                            <div>
                                <div>{extension} {size}</div>
                                <small>{file_name} ({duration})</small>
                                <form action="{update_usage_url}" method="post">
                                    <input name="file_name" type="hidden" value="{file_name}">
                                    <select aria-label="{t_usage_restrictions}" class="usage" name="usage">
                                        {usage_options}
                                    </select>
                                </form>
                            </div>
                            <div>
                                {delete_form}
                            </div>
                        </div>
                        <div>
                            {r_browser_support}
                        </div>
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_audio_files = translations.audio_files;
        formatdoc!(r#"
            <h2>{t_audio_files}</h2>

            {duration_info}

            <ul class="versions">
                {audio_files_rendered}
            </ul>
        "#)
    };

    let video_files = if video.video_files.is_empty() {
        String::new()
    } else {
        let video_files = video.video_files_sorted_by_fidelity(Desc);

        let mut duration_range = None;

        if video_files.len() > 1 {
            for video_file in &video_files {
                if let Some(video_meta) = &video_file.video_meta {
                    match &mut duration_range {
                        Some((shortest_video_file, longest_video_file)) => {
                            if video_meta.duration > *longest_video_file {
                                *longest_video_file = video_meta.duration;
                            }

                            if video_meta.duration > *shortest_video_file {
                                *shortest_video_file = video_meta.duration;
                            }
                        }
                        None => {
                            duration_range = Some((video_meta.duration, video_meta.duration));
                        }
                    }
                }
            }
        }

        let duration_info = if let Some((shortest_video_file, longest_video_file)) = duration_range {
            if longest_video_file - shortest_video_file < 1.0 {
                String::new()
            } else {
                let longest = precise_duration(longest_video_file);
                let shortest = precise_duration(shortest_video_file);

                let t_not_ok = translations.not_ok;
                let t_length_diverges_between_different_formats = translations.length_diverges_between_different_formats;
                let failure_icon = hyper8_icons::failure(t_not_ok);
                format!(r#"<div class="error">{failure_icon} {t_length_diverges_between_different_formats} ({shortest} - {longest})</div>"#)
            }
        } else {
            String::new()
        };

        let video_files_rendered = video_files
            .iter()
            .map(|video_file| {
                let attachment_type;
                let label;
                let r_browser_support;
                let r_video_meta;

                match &video_file.video_meta {
                    Some(video_meta) => {
                        let duration = precise_duration(video_meta.duration);
                        let frame_rate = video_meta.video_frame_rate;
                        let height = video_meta.height;
                        let size = human_size(video_file.file_meta.size);
                        let width = video_meta.width;

                        let frame_rate_formatted = if frame_rate.fract() > f32::EPSILON {
                            format!("{frame_rate:.3}")
                        } else {
                            format!("{frame_rate}")
                        };

                        let reasons;
                        let support;

                        match BrowserSupport::video_analysis(
                            &video_file.file_name,
                            translations,
                            video_meta
                        ) {
                            BrowserSupport::Supported(verdict) => {
                                attachment_type = "attached_success";
                                reasons = verdict.explanation;
                                support = format!("{} – {}", translations.good_browser_support, verdict.summary);
                            }
                            BrowserSupport::Unknown => {
                                attachment_type = "attached_warning";
                                reasons = translations.hyper_8_could_not_determine_whether_this_file_format_is_generally_compatible_with_browsers_or_not.to_string();
                                support = translations.browser_support_unclear.to_string();
                            }
                            BrowserSupport::Unsupported(verdict) => {
                                attachment_type = "attached_error";
                                reasons = verdict.explanation;
                                support = format!("{} – {}", translations.not_supported_by_browsers, verdict.summary);
                            }
                        }

                        label = format!("{height}p {size}");
                        r_browser_support = format!(
                            r#"{support} <span class="reasons" title="{reasons}">🛈</span>"#
                        );
                        r_video_meta = format!("{width}×{height}, {frame_rate_formatted}fps, {duration}");
                    }
                    None => {
                        attachment_type = "attached_info";
                        label = String::new();
                        r_browser_support = String::new();
                        r_video_meta = translations.metadata_pending.to_string();
                    }
                }

                let file_name = &video_file.file_name;

                let t_delete = translations.delete;
                let t_really_delete = translations.really_delete;
                let delete_icon = hyper8_icons::failure(t_delete);
                let delete_video_file_action = routes::video_delete_video_file(video);
                let delete_form = formatdoc!(r#"
                    <form
                    action="{delete_video_file_action}"
                    data-confirm
                    data-confirm-label="{t_delete}"
                    data-confirm-text="{t_really_delete}"
                    method="post">
                        <input name="file_name" type="hidden" value="{file_name}">
                        <button>{delete_icon}</button>
                    </form>
                "#);

                let usage_options = MediaUsage::variants(translations)
                    .iter()
                    .map(|(label, usage)| {
                        let key = usage.key();
                        let selected = if *usage == video_file.usage { "selected" } else { "" };

                        format!(r#"<option {selected} value="{key}">{label}</option>"#)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                let update_usage_url = routes::video_update_video_file_usage(video);

                formatdoc!(r#"
                    <li>
                        <div class="actionable_item attachment {attachment_type}">
                            <div>
                                <div>{label}</div>
                                <small>{file_name} ({r_video_meta})</small>
                                <form action="{update_usage_url}" method="post">
                                    <input name="file_name" type="hidden" value="{file_name}">
                                    <select aria-label="{t_usage_restrictions}"class="usage" name="usage">
                                        {usage_options}
                                    </select>
                                </form>
                            </div>
                            <div>
                                {delete_form}
                            </div>
                        </div>
                        <div>
                            {r_browser_support}
                        </div>
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let t_video_files = translations.video_files;
        formatdoc!(r#"
            <h2>{t_video_files}</h2>

            {duration_info}

            <ul class="versions">
                {video_files_rendered}
            </ul>
        "#)
    };

    let upload_audio_video_url = routes::upload_audio_video(&video.site_path);

    let t_upload_video_or_audio_file = translations.upload_video_or_audio_file;
    let upload_audio_video_button = upload_widget(&upload_audio_video_url, t_upload_video_or_audio_file);

    let render_additional_formats = additional_formats_form(translations, video);

    let video_site_path = &video.site_path.normalized();
    let body = formatdoc!(r#"
        <div class="local_processing" data-media-jobs="{video_site_path}"></div>
        <div class="form_group">
            {video_files}
        </div>
        <div class="form_group">
            {audio_files}
        </div>
        <div class="form_group">
            {upload_audio_video_button}
        </div>

        {render_additional_formats}
    "#);

    let t_video_files = translations.video_files;
    let video_title = video.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_video_files} – {video_title}");
    let html = layout(
        &body,
        context,
        Location::media(&video.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

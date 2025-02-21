// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;

use indoc::formatdoc;

use crate::{
    Context,
    Field,
    Language,
    Translations,
    Video
};
use crate::browser_support;
use crate::editor::{Checkbox, Input, Select, Textarea};
use crate::editor::endpoints;
use crate::editor::widgets::{
    add_link_form,
    layout,
    links,
    resource_actions,
    upload_widget
};
use crate::icons;
use crate::util::{precise_duration, human_size};

mod add_link;
mod delete;
mod delete_link;
mod delete_poster;
mod delete_version;
mod downsize;
mod edit;
mod r#move;
mod poster;
mod random_poster;
mod update;
mod update_poster;
mod upload_poster;
mod upload_video;

pub use add_link::add_link;
pub use delete::delete;
pub use delete_link::delete_link;
pub use delete_poster::delete_poster;
pub use delete_version::delete_version;
pub use downsize::downsize;
pub use edit::edit;
pub use r#move::r#move;
pub use poster::poster;
pub use random_poster::random_poster;
pub use update::update;
pub use update_poster::update_poster;
pub use upload_poster::upload_poster;
pub use upload_video::upload_video;

pub enum VideoFormFeedback<'a> {
    Move {
        slug: Field
    },
    Site(endpoints::site::SiteFormFeedback),
    Update {
        copy_link: Option<bool>,
        description: &'a str,
        download: Option<bool>,
        embedding: Option<bool>,
        offline: bool,
        release_date: Field,
        sort_number: Field,
        title: Field,
        unlisted: bool
    },
    UploadPoster {
        error: String
    }
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

    let path = &video.path;

    let t_advanced_settings = translations.advanced_settings;
    let t_best = translations.best;
    let t_quality = translations.quality;
    let t_render_additional_format = translations.render_additional_format;
    let t_render_format = translations.render_format;
    let t_worst = translations.worst;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_render_additional_format}</h2>
            <form action="/downsize/{path}" method="post">
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

pub fn edit_page(
    context: &Context,
    form_feedback: Option<VideoFormFeedback>,
    language: &Language,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let parent_options = video.parent_options(&site);
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

    let mut input_offline = Checkbox::new("offline", translations.offline);
    let mut input_release_date = Input::date("release_date", translations.release_date);
    let mut input_sort_number = Input::number("sort_number", translations.sort_number);
    let mut input_title = Input::text("title", translations.title);
    let mut input_unlisted = Checkbox::new("unlisted", translations.unlisted);
    let mut select_copy_link = Select::new("copy_link", translations.copy_link);
    let mut select_download = Select::new("download", translations.downloads);
    let mut select_embedding = Select::new("embedding", translations.embedding);
    let mut textarea_description = Textarea::markdown_subset(language, "description", translations.description);

    let t_disabled = translations.disabled;
    let t_enabled = translations.enabled;
    let t_inherit = translations.inherit;

    let t_copy_link_disabled_enabled = if parent_options.copy_link { t_enabled } else { t_disabled };
    select_copy_link.option("", &format!("{t_inherit} ({t_copy_link_disabled_enabled})"));
    select_copy_link.option("enabled", t_enabled);
    select_copy_link.option("disabled", t_disabled);

    let t_download_disabled_enabled = if parent_options.download { t_enabled } else { t_disabled };
    select_download.option("", &format!("{t_inherit} ({t_download_disabled_enabled})"));
    select_download.option("enabled", t_enabled);
    select_download.option("disabled", t_disabled);

    let t_embedding_disabled_enabled = if parent_options.embedding { t_enabled } else { t_disabled };
    select_embedding.option("", &format!("{t_inherit} ({t_embedding_disabled_enabled})"));
    select_embedding.option("enabled", t_enabled);
    select_embedding.option("disabled", t_disabled);

    textarea_description.id("video_description");

    let feedback_update = if let Some(VideoFormFeedback::Update {
        copy_link,
        description,
        download,
        embedding,
        offline,
        release_date,
        sort_number,
        title,
        unlisted
    }) = &form_feedback {
        let copy_link_value = match copy_link {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download_value = match download {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding_value = match embedding {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        input_offline.checked(*offline);
        input_release_date.validated_value(release_date);
        input_sort_number.validated_value(sort_number);
        input_title.validated_value(title);
        input_unlisted.checked(*unlisted);
        select_copy_link.selected(copy_link_value);
        select_download.selected(download_value);
        select_embedding.selected(embedding_value);
        textarea_description.value(description);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let copy_link = match video.copy_link {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let download = match video.download {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };

        let embedding = match video.embedding {
            Some(true) => "enabled",
            Some(false) => "disabled",
            None => ""
        };
        let release_date = video.release_date.as_ref().map(|date| date.to_string());
        let sort_number = video.sort_number.as_ref().map(|number| number.to_string());

        input_offline.checked(video.offline);
        input_release_date.value_option(&release_date);
        input_sort_number.value_option(&sort_number);
        input_title.value_option(&video.title);
        input_unlisted.checked(video.unlisted);
        select_copy_link.selected(copy_link);
        select_download.selected(download);
        select_embedding.selected(embedding);
        textarea_description.value_option(&video.description);

        String::new()
    };

    let slug_form = if path == "" {
        String::new()
    } else {
        let mut input_permalink = Input::permalink("slug", video.parent_path(), &site, translations);

        input_permalink.required();

        let feedback = if let Some(VideoFormFeedback::Move { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(video.slug());
            String::new()
        };

        let t_a_permalink_forms_part_of_text = translations.a_permalink_forms_part_of_text;
        let t_update_permalink = translations.update_permalink;
        formatdoc!(r#"
            <p>
                {t_a_permalink_forms_part_of_text}
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

    let r_resource_actions = resource_actions(context, video.offline, path, &site, translations);

    let video_icon = icons::video(translations.video);
    let video_title = video.label(translations);

    let t_danger_zone = translations.danger_zone;
    let t_delete = translations.delete;
    let t_delete_video = translations.delete_video;
    let t_deleting_the_video_text = translations.deleting_the_video_text;
    let t_poster_image = translations.poster_image;
    let t_really_delete = translations.really_delete;
    let t_update_video = translations.update_video;
    let body = formatdoc!(r#"
        {errors}

        <div class="form_group">
            <h2 class="with_button">
                {video_icon}
                <span>{video_title}</span>
            </h2>
            {r_resource_actions}
        </div>

        <div class="form_group">
            <form action="/video/{path}" autocomplete="off" data-warn-discard method="post">
                {input_title}
                {textarea_description}
                <div class="form_split divide_3_1">
                    {input_release_date}
                    {input_sort_number}
                </div>
                <div class="form_split even">
                    {select_copy_link}
                    {select_download}
                    {select_embedding}
                </div>
                {input_offline}
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

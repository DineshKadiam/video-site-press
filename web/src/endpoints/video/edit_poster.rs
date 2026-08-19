// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{
    AspectRatio,
    SitePath,
    Video
};

use crate::{
    Context,
    Layout,
    Location,
    SessionState,
    Textarea
};
use crate::routes;
use crate::widgets::{not_found, upload_widget};

pub struct UploadPosterFormFeedback {
    pub error: String
}

pub async fn edit_poster(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_video(&site_path) {
        Some(video) => {
            edit_poster_page(
                &context,
                None,
                &session_state,
                &video
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn edit_poster_page(
    context: &Context,
    form_feedback: Option<UploadPosterFormFeedback>,
    session_state: &SessionState,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;

    let pick_poster_action = routes::video_pick_poster(video);

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

            let mut description = Textarea::image_description(
                "description",
                translations.image_description_for_blind_or_visually_impaired_people
            );

            description
                .id("poster_description")
                .value_option(&poster.description);

            let t_delete_poster = translations.delete_poster;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;

            let delete_poster_action = routes::video_delete_poster(video);
            let update_poster_action = routes::video_update_poster(video);

            let thumbnail_aspect_numeric = site.poster_aspect.numeric();
            let thumbnail_aspect_representation = site.poster_aspect.representation();

            let video_aspect_numeric = video.highest_quality_video_file()
                .and_then(|video_file| video_file.video_meta.as_ref())
                .map(|video_meta| video_meta.aspect_ratio())
                .unwrap_or(thumbnail_aspect_numeric);
            let video_aspect_representation = AspectRatio::approximate_representation(video_aspect_numeric);

            let poster_format = if let Some(poster_meta) = &poster.poster_meta {
                let background = poster_meta.lqip.gradient_radial();
                let poster_aspect_numeric = poster_meta.aspect_ratio();
                let poster_aspect_representation = AspectRatio::approximate_representation(poster_aspect_numeric);

                let t_poster_format = translations.poster_format;
                formatdoc!(r#"
                    <div>
                        <span class="dot" style="{background}"></span> {t_poster_format} ({poster_aspect_representation})
                    </div>
                "#)
            } else {
                String::new()
            };

            let t_thumbnail_format = translations.thumbnail_format;
            let t_video_format = translations.video_format;
            formatdoc!(r#"
                <style>
                    :root {{
                        --poster-aspect: {thumbnail_aspect_numeric};
                        --video-aspect: {video_aspect_numeric};
                    }}
                </style>
                <div aria-hidden="true" class="poster_preview_outer">
                    <div class="video_frame_outer">
                        <div class="video_frame"></div>
                        <div class="poster_frame">
                            <div class="thumbnail_frame_outer">
                                <div class="thumbnail_frame"></div>
                            </div>
                            <img {alt} src="{src}">
                        </div>
                    </div>
                </div>
                <div>{file_name}</div>
                <div class="poster_preview_legend">
                    <div>
                        <span class="dot video_frame"></span> {t_video_format} ({video_aspect_representation})
                    </div>
                    {poster_format}
                    <div>
                        <span class="dot thumbnail_frame"></span> {t_thumbnail_format} ({thumbnail_aspect_representation})
                    </div>
                </div>
                <div class="button_group">
                    {poster_upload_button}
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

    let t_poster_image = translations.poster_image;

    let pick_from_video = if let Some(video_file) = video.highest_quality_video_file() {
        let src = routes::video_stream(&video_file.file_name, video);

        let t_jump_to_a_random_frame = translations.jump_to_a_random_frame;
        let t_pick_a_poster_from_the_video = translations.pick_a_poster_from_the_video;
        let t_use_current_frame_as_poster = translations.set_current_frame_as_poster;
        formatdoc!(r#"
            <h2>{t_pick_a_poster_from_the_video}</h2>

            <video controls preload="auto" src="{src}"></video>

            <form action="{pick_poster_action}" method="post">
                <input name="timecode" type="hidden" value="0">
                <div class="button_group">
                    <div style="align-items: center; column-gap: .5rem; display: flex;">
                        <button class="action">{t_use_current_frame_as_poster}</button>
                    </div>
                    <button class="action" id="random_frame">{t_jump_to_a_random_frame}</button>
                </div>
            </form>

            <script>
                const randomFrameButton = document.querySelector('#random_frame');
                const timecode = document.querySelector('input[name="timecode"');
                const video = document.querySelector('video');

                randomFrameButton.addEventListener('click', event => {{
                    video.currentTime = Math.random() * video.duration;
                    event.preventDefault();
                }});

                video.addEventListener('timeupdate', () => {{
                    timecode.setAttribute('value', video.currentTime.toString());
                }});
            </script>
        "#)
    } else {
        String::new()
    };

    let body = formatdoc!(r#"
        <h2>{t_poster_image}</h2>

        <div class="form_group">
            {poster_form}
        </div>

        {pick_from_video}
    "#);

    let t_poster_image = translations.poster_image;
    let video_title = video.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_poster_image} – {video_title}");

    let layout = Layout::new();

    let html = layout.render(
        &body,
        context,
        Location::poster(&video.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

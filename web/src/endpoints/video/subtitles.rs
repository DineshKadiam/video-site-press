// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::SitePath;
use hyper8_util::html_escape_outside_attribute;

use crate::{Context, Location, SessionState};
use crate::routes;
use crate::widgets::{layout, not_found, upload_widget};

pub async fn subtitles(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;

    let video = match context.get_video(&site_path) {
        Some(video) => video,
        None => return not_found(&context, &session_state, &site_path)
    };

    let site = context.get_site();

    let head_title = video.title_or_slug_or_generic_label(translations);

    let t_subtitles = translations.subtitles;

    let upload_url = routes::video_upload_subtitles(&video);
    let subtitles_upload_button = upload_widget(&upload_url, translations.upload_new_subtitles);

    let subtitles_list = video.subtitle_files
        .iter()
        .map(|subtitle_file| {
            let file_name = html_escape_outside_attribute(&subtitle_file.file_name);
            let label = html_escape_outside_attribute(&subtitle_file.label);

            let language = match &subtitle_file.language {
                Some(code) => code,
                None => translations.no_language_info
            };

            let t_delete = translations.delete;
            let t_really_delete = translations.really_delete;
            let delete_icon = hyper8_icons::failure(t_delete);
            let delete_version_action = routes::video_delete_subtitles(&video);
            let delete_form = formatdoc!(r#"
                <form
                action="{delete_version_action}"
                data-confirm
                data-confirm-label="{t_delete}"
                data-confirm-text="{t_really_delete}"
                method="post">
                    <input name="file_name" type="hidden" value="{file_name}">
                    <button>{delete_icon}</button>
                </form>
            "#);

            let attachment;
            let attachment_class;
            if let Err(err) = &subtitle_file.subtitles {
                attachment = format!(r#"<div>{err}</div>"#);
                attachment_class = "attachment attached_error";
            } else {
                attachment = String::new();
                attachment_class = "";
            }

            let edit_icon = hyper8_icons::edit(Some(translations.edit));
            let edit_subtitles_url = routes::video_edit_subtitles(&subtitle_file.file_name, &video);
            formatdoc!(r#"
                <div class="actionable_item {attachment_class}">
                    <div>
                        <div>{label} ({language})</div>
                        <small>{file_name}</small>
                    </div>
                    <div>
                        <a href="{edit_subtitles_url}">
                            {edit_icon}
                        </a>
                        {delete_form}
                    </div>
                </div>
                {attachment}
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let body = formatdoc!(r#"
        <h2>{t_subtitles}</h2>

        <div style="margin-block: 1rem;">
            {subtitles_list}
        </div>

        <div class="form_group">
            {subtitles_upload_button}
        </div>
    "#);

    let html = layout(
        &body,
        &context,
        Location::subtitles(&video.site_path),
        &session_state,
        &site,
        head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

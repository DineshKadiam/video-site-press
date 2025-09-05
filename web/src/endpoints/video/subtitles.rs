// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::SitePath;
use hyper8_core::util::html_escape_outside_attribute;

use crate::{Context, Location, Session};
use crate::routes;
use crate::widgets::{layout, not_found, upload_widget};

pub async fn subtitles(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session = Session::new(&session);
    let translations = &session.language.translations;

    let video = match context.get_video(&site_path) {
        Some(video) => video,
        None => return not_found(&context, &session, &site_path)
    };

    let site = context.get_site();

    let head_title = video.title_or_slug_or_generic_label(translations);

    let t_subtitles = translations.subtitles;

    let upload_url = routes::video_upload_subtitles(&video);
    let subtitles_upload_button = upload_widget(&upload_url, translations.upload_new_subtitles);

    let subtitles_list = video.subtitles
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
                method="post"
                style="display: inline;">
                    <input name="file_name" type="hidden" value="{file_name}">
                    <button class="danger delete_button">{delete_icon}</button>
                </form>
            "#);

            let edit_subtitles_url = routes::video_edit_subtitles(&subtitle_file.file_name, &video);

            formatdoc!(r#"
                <div>
                    <a href="{edit_subtitles_url}" style="text-decoration: underline;">{label}</a> ({file_name}, {language}) {delete_form}
                </div>
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
        &session,
        &site,
        head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use crate::{Context, SitePath};
use crate::editor::Location;
use crate::editor::routes;
use crate::editor::widgets::{layout, not_found, upload_widget};
use crate::icons;

// TODO: Allow user to specify language or label for each subtitle track.
//       Possibly also possibility to order or specify a default track.
pub async fn subtitles(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();
    let translations = &language.translations;

    let video = match context.get_video(&site_path) {
        Some(video) => video,
        None => return not_found(&context, &language, &site_path)
    };

    let site = context.get_site();

    let head_title = video.title_or_slug_or_generic_label(translations);

    let t_subtitles = translations.subtitles;

    let upload_url = routes::video_upload_subtitles(&video);
    let subtitles_upload_button = upload_widget(&upload_url, translations.upload_new_subtitles);

    let subtitles_list = video.subtitles
        .iter()
        .enumerate()
        .map(|(index, subtitle_file)| {
            let file_name = &subtitle_file.file_name;
            // TODO: label needs to be escaped
            let label = match &subtitle_file.language {
                Some(code) => code,
                None => &subtitle_file.file_name
            };

            let t_delete = translations.delete;
            let t_really_delete = translations.really_delete;
            let delete_icon = icons::failure(t_delete);
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

            formatdoc!(r#"
                <div>
                    {label} ({index}.vtt) {delete_form}
                </div>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let body = formatdoc!(r#"
        <h2>{t_subtitles}</h2>

        {subtitles_list}

        <div class="form_group">
            {subtitles_upload_button}
        </div>
    "#);

    let html = layout(
        &body,
        &context,
        &language,
        Location::subtitles(&video.site_path),
        &site,
        head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

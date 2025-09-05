// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{SitePath, Video};

use crate::{
    Context,
    Field,
    Input,
    Location,
    Session,
    Textarea
};
use crate::routes;
use crate::widgets::{error_page, layout, not_found};

pub struct UpdateFormFeedback<'a> {
    pub content: &'a str,
    pub label: Field,
    pub language: Field
}

pub async fn edit_subtitles(
    context: Data<Arc<Context>>,
    path: ActixPath<(String, String)>,
    session: ActixSession
) -> HttpResponse {
    let (file_name, site_path_str) = path.into_inner();
    let site_path = SitePath::new(site_path_str);

    let session = Session::new(&session);

    match context.get_video(&site_path) {
        Some(video) => edit_subtitles_page(&context, &file_name, None, &session, &video),
        None => not_found(&context, &session, &site_path)
    }
}

pub fn edit_subtitles_page(
    context: &Context,
    file_name: &str,
    form_feedback: Option<UpdateFormFeedback>,
    session: &Session,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session.language.translations;
    let t_not_saved = translations.not_saved;

    let subtitles = match video.subtitles
        .iter()
        .find(|subtitle_file| subtitle_file.file_name == file_name) {
        Some(subtitle_file) => subtitle_file,
        None => {
            let err = format!("The subtitle file '{file_name}' was not found.");
            return error_page(context, &err, session)
        }
    };

    let mut input_label = Input::text("label", translations.label);
    let mut input_language = Input::text("language", translations.language_label);
    let mut textarea_content = Textarea::manifest("content", translations.content);

    let feedback_update = if let Some(UpdateFormFeedback {
        content,
        label,
        language
    }) = &form_feedback {
        input_label.validated_value(label);
        input_language.validated_value(language);
        textarea_content.value(content);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        input_label.value(&subtitles.label);
        input_language.value_option(&subtitles.language);

        let subtitle_file_path = context.site_dir
            .join(video.site_path.filesystem_relative())
            .join(&subtitles.file_name);

        let content = fs::read_to_string(subtitle_file_path).unwrap();

        textarea_content.value(&content);

        String::new()
    };

    let t_update_subtitles = translations.update_subtitles;

    let update_action = routes::video_edit_subtitles(file_name, video);

    let body = formatdoc!(r#"
        <h2>{file_name}</h2>
        <div class="form_group">
            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                {input_label}
                {input_language}
                {textarea_content}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_subtitles}</button> {feedback_update}
                </div>
            </form>
        </div>
    "#);

    let head_title = &subtitles.label;
    let html = layout(
        &body,
        context,
        Location::subtitles(&video.site_path),
        session,
        &site,
        head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

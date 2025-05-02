// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};

use crate::{Context, SitePath};
use crate::editor::Location;
use crate::editor::widgets::{layout, not_found};

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

    let body = format!("<h2>{t_subtitles}</h2>");

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

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{Context, Language, Video};
use crate::editor::Location;
use crate::editor::routes;
use crate::editor::widgets::layout;

pub fn pending_manifest_errors(
    context: &Context,
    language: &Language,
    video: &Video
) -> HttpResponse {
    let translations = &language.translations;
    let t_pending_manifest_errors = translations.pending_manifest_errors;

    let video_title = video.title_or_slug_or_generic_label(translations);

    let manifest_path = context.site_dir
        .join(video.site_path.filesystem_relative())
        .join("video.eno");

    let manifest_url = routes::video_manifest(video);

    let t_updating_video_from_editor_currently_not_allowed =
        translations.updating_video_from_editor_currently_not_allowed(
            video_title,
            &manifest_path.to_string_lossy()
        );

    let manifest_errors = video.manifest_errors
        .iter()
        .map(|error| {
            format!(r#"<pre class="error">{error}</pre>"#)
        })
        .collect::<Vec<String>>()
        .join("");

    let t_resolve_the_issue_on_disk_or_on_manifest_page =
        translations.resolve_the_issue_on_disk_or_on_manifest_page(&manifest_url);

    let body = formatdoc!(r#"
        <h2>{t_pending_manifest_errors}</h2>
        <p>
            {t_updating_video_from_editor_currently_not_allowed}
        </p>
        {manifest_errors}
        <p>
            {t_resolve_the_issue_on_disk_or_on_manifest_page}
        </p>
    "#);

    let site = &context.get_site();

    let html = layout(
        &body,
        context,
        language,
        Location::AuxiliaryPage,
        site,
        t_pending_manifest_errors
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

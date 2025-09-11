// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use hyper8_core::SitePath;
use hyper8_core::util::string_from_pathbuf;

use crate::{
    Context,
    Location,
    SessionState,
    Textarea
};
use crate::routes;
use crate::widgets::layout;

pub async fn edit_manifest(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.editor_state, &session);
    edit_manifest_page(&context, &session_state)
}

pub fn edit_manifest_page(
    context: &Context,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;

    let manifest_errors = if site.manifest_errors.is_empty() {
        String::new()
    } else {
        site.manifest_errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let manifest_path = context.site_dir.join("site.eno");

    let content = fs::read_to_string(&manifest_path).unwrap();
    let manifest_path_str = string_from_pathbuf(manifest_path);

    let mut textarea_content = Textarea::manifest("content", &manifest_path_str);

    textarea_content.value(&content);

    let update_action = routes::SITE_MANIFEST;

    let t_site_manifest = translations.site_manifest;
    let t_update_manifest = translations.update_manifest;
    let body = formatdoc!(r#"
        <h2>{t_site_manifest}</h2>

        {manifest_errors}

        <div class="form_group">
            <form action="{update_action}" autocomplete="off" data-warn-discard method="post">
                {textarea_content}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_manifest}</button>
                </div>
            </form>
        </div>
    "#);

    let html = layout(
        &body,
        context,
        Location::site_manifest(&SitePath::root_path()),
        session_state,
        &site,
        site.title()
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

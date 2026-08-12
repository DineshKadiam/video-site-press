// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Collection, SitePath};
use hyper8_util::string_from_pathbuf;

use crate::actix::html_response;
use crate::{
    Context,
    Location,
    SessionState,
    Textarea
};
use crate::routes;
use crate::widgets::{layout, not_found};

pub async fn edit_manifest(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_collection(&site_path) {
        Some(collection) => edit_manifest_page(&collection, &context, &session_state),
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn edit_manifest_page(
    collection: &Collection,
    context: &Context,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;

    let manifest_errors = if collection.manifest_errors.is_empty() {
        String::new()
    } else {
        collection.manifest_errors
            .iter()
            .map(|error| {
                format!(r#"<pre class="error">{error}</pre>"#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let manifest_path = context.site_dir
        .join(collection.site_path.filesystem_relative())
        .join("collection.eno");

    let content = match manifest_path.exists() {
        true => fs::read_to_string(&manifest_path).unwrap(),
        false => String::new()
    };

    let manifest_path_str = string_from_pathbuf(manifest_path);

    let mut textarea_content = Textarea::manifest("content", &manifest_path_str);

    textarea_content.value(&content);

    let update_action = routes::collection_manifest(collection);

    let t_collection_manifest = translations.collection_manifest;
    let t_update_manifest = translations.update_manifest;
    let body = formatdoc!(r#"
        <h2>{t_collection_manifest}</h2>

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

    let collection_title = collection.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_collection_manifest} – {collection_title}");
    let html = layout(
        &body,
        context,
        Location::manifest(&collection.site_path),
        session_state,
        &site,
        &head_title
    );

    html_response(html)
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Language,
    SitePath
};
use crate::editor::{Location, Textarea};
use crate::editor::routes;
use crate::editor::widgets::{layout, not_found};
use crate::util::string_from_pathbuf;

pub async fn edit_manifest(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_collection(&site_path) {
        Some(collection) => edit_manifest_page(&collection, &context, &language),
        None => not_found(&context, &language, &site_path)
    }
}

pub fn edit_manifest_page(
    collection: &Collection,
    context: &Context,
    language: &Language
) -> HttpResponse {
    let site = context.get_site();
    let translations = &language.translations;

    let errors = if collection.errors.is_empty() {
        String::new()
    } else {
        collection.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let manifest_path = context.site_dir
        .join(collection.site_path.filesystem_relative())
        .join("collection.eno");

    let content = fs::read_to_string(&manifest_path).unwrap();
    let manifest_path_str = string_from_pathbuf(manifest_path);

    let mut textarea_content = Textarea::manifest("content", &manifest_path_str);

    textarea_content.value(&content);

    let update_action = routes::collection_manifest(collection);

    let t_collection_manifest = translations.collection_manifest;
    let t_update_manifest = translations.update_manifest;
    let body = formatdoc!(r#"
        <h2>{t_collection_manifest}</h2>

        {errors}

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
        language,
        Location::manifest(&collection.site_path),
        &site,
        collection.title_or_slug_or_generic_label(translations)
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

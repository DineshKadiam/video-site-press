// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{
    Context,
    Language,
    SitePath
};
use crate::editor::{Location, Textarea};
use crate::editor::routes;
use crate::editor::widgets::layout;
use crate::util::string_from_pathbuf;

pub async fn edit_manifest(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();
    edit_manifest_page(&context, &language)
}

pub fn edit_manifest_page(context: &Context, language: &Language) -> HttpResponse {
    let site = context.get_site();
    let translations = &language.translations;

    let errors = if site.errors.is_empty() {
        String::new()
    } else {
        site.errors
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
        Location::site_manifest(&SitePath::root_path()),
        &site,
        site.title()
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

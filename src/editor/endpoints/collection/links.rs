// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Language,
    SitePath
};
use crate::editor::actix::html_response;
use crate::editor::Location;
use crate::editor::routes;
use crate::editor::widgets::{
    add_link_form,
    layout,
    links as links_widget,
    not_found
};

pub async fn links(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_collection(&site_path) {
        Some(collection) => edit_links_page(&collection, &context, &language),
        None => not_found(&context, &language, &site_path)
    }
}

pub fn edit_links_page(
    collection: &Collection,
    context: &Context,
    language: &Language
) -> HttpResponse {
    let site = context.get_site();
    let translations = &language.translations;

    let create_link_form = add_link_form(&routes::collection_add_link(collection), translations);
    let links_form = links_widget(&collection.links, &routes::collection_delete_link(collection), translations);

    let body = formatdoc!(r#"
        {links_form}
        {create_link_form}
    "#);

    let t_links = translations.links;
    let collection_title = collection.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_links} – {collection_title}");
    let html = layout(
        &body,
        context,
        language,
        Location::links(&collection.site_path),
        &site,
        &head_title
    );

    html_response(html)
}

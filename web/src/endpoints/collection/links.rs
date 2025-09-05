// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Collection, SitePath};

use crate::actix::html_response;
use crate::{Context, Location, Session};
use crate::routes;
use crate::widgets::{
    add_link_form,
    layout,
    links as links_widget,
    not_found
};

pub async fn links(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session = Session::new(&session);

    match context.get_collection(&site_path) {
        Some(collection) => edit_links_page(&collection, &context, &session),
        None => not_found(&context, &session, &site_path)
    }
}

pub fn edit_links_page(
    collection: &Collection,
    context: &Context,
    session: &Session
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session.language.translations;

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
        Location::links(&collection.site_path),
        session,
        &site,
        &head_title
    );

    html_response(html)
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Playlist, SitePath};

use crate::{Context, Location, SessionState};
use crate::routes;
use crate::widgets::{
    add_link_form,
    layout,
    links as links_widget,
    not_found
};

pub async fn links(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.editor_state, &session);

    match context.get_playlist(&site_path) {
        Some(playlist) => edit_links_page(&context, &playlist, &session_state),
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn edit_links_page(
    context: &Context,
    playlist: &Playlist,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;

    let create_link_form = add_link_form(&routes::playlist_add_link(playlist), translations);
    let links_form = links_widget(&playlist.links, &routes::playlist_delete_link(playlist), translations);

    let body = formatdoc!(r#"
        {links_form}
        {create_link_form}
    "#);

    let t_links = translations.links;
    let playlist_title = playlist.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_links} – {playlist_title}");
    let html = layout(
        &body,
        context,
        Location::links(&playlist.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use hyper8_core::Playlist;

use crate::{Context, Location, SessionState};
use crate::routes;
use crate::widgets::layout;

pub fn pending_manifest_errors(
    context: &Context,
    playlist: &Playlist,
    session_state: &SessionState
) -> HttpResponse {
    let translations = &session_state.language.translations;
    let t_pending_manifest_errors = translations.pending_manifest_errors;

    let playlist_title = playlist.title_or_slug_or_generic_label(translations);

    let manifest_path = context.site_dir
        .join(playlist.site_path.filesystem_relative())
        .join("playlist.eno");

    let manifest_url = routes::playlist_manifest(playlist);

    let t_updating_playlist_from_editor_currently_not_allowed =
        translations.updating_playlist_from_editor_currently_not_allowed(
            playlist_title,
            &manifest_path.to_string_lossy()
        );

    let manifest_errors = playlist.manifest_errors
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
            {t_updating_playlist_from_editor_currently_not_allowed}
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
        Location::AuxiliaryPage,
        session_state,
        site,
        t_pending_manifest_errors
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

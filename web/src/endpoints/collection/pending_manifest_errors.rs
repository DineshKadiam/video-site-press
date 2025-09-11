// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;

use indoc::formatdoc;

use hyper8_core::Collection;

use crate::{Context, Location, SessionState};
use crate::actix::html_response;
use crate::routes;
use crate::widgets::layout;

pub fn pending_manifest_errors(
    collection: &Collection,
    context: &Context,
    session_state: &SessionState
) -> HttpResponse {
    let translations = &session_state.language.translations;
    let t_pending_manifest_errors = translations.pending_manifest_errors;

    let collection_title = collection.title_or_slug_or_generic_label(translations);

    let manifest_path = context.site_dir
        .join(collection.site_path.filesystem_relative())
        .join("collection.eno");

    let manifest_url = routes::collection_manifest(collection);

    let t_updating_collection_from_editor_currently_not_allowed =
        translations.updating_collection_from_editor_currently_not_allowed(
            collection_title,
            &manifest_path.to_string_lossy()
        );

    let manifest_errors = collection.manifest_errors
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
            {t_updating_collection_from_editor_currently_not_allowed}
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

    html_response(html)
}

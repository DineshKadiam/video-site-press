// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use indoc::formatdoc;

use hyper8_core::Site;

use crate::{Context, Location, Session};
use crate::actix::html_response;
use crate::routes;
use crate::widgets::layout;

pub fn pending_manifest_errors(
    context: &Context,
    session: &Session,
    site: &Site
) -> HttpResponse {
    let translations = &session.language.translations;
    let t_pending_manifest_errors = translations.pending_manifest_errors;

    let manifest_path = context.site_dir
        .join("site.eno");

    let manifest_url = routes::SITE_MANIFEST;

    let t_updating_site_from_editor_currently_not_allowed =
        translations.updating_site_from_editor_currently_not_allowed(
            &manifest_path.to_string_lossy()
        );

    let manifest_errors = site.manifest_errors
        .iter()
        .map(|error| {
            format!(r#"<pre class="error">{error}</pre>"#)
        })
        .collect::<Vec<String>>()
        .join("");

    let t_resolve_the_issue_on_disk_or_on_manifest_page =
        translations.resolve_the_issue_on_disk_or_on_manifest_page(manifest_url);

    let body = formatdoc!(r#"
        <h2>{t_pending_manifest_errors}</h2>
        <p>
            {t_updating_site_from_editor_currently_not_allowed}
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
        session,
        site,
        t_pending_manifest_errors
    );

    html_response(html)
}

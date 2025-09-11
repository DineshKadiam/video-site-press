// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, Location, SessionState};
use crate::widgets::layout;

/// Page on which deployment progress is reported (updated in real time via
/// client scripting)
pub async fn deployment(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.editor_state, &session);
    let site = context.get_site();

    let translations = &session_state.language.translations;

    let mut body = String::new();

    let t_deployment = translations.deployment;
    body.push_str(&format!("<h2>{t_deployment}</h2>"));

    if let Some(hyper_url) = &site.base_url {
        let url = &hyper_url.representation;
        let link = format!(r#"<a href="{url}" target="_blank">{url}</a>"#);
        let t_files_are_being_deployed_to_xxx = translations
            .files_are_being_deployed_to_xxx(&link);

        let text = format!("<p>{t_files_are_being_deployed_to_xxx}</p>");
        body.push_str(&text);
    }

    if let Ok(deployment) = context.deploying.lock() {
        let feedback = &deployment.feedback;

        let ellipsis = if deployment.active() { "\n…" } else { "" };

        let wrapped = formatdoc!(r#"
            <pre id="deployment_feedback">
                {feedback}{ellipsis}
            </pre>
        "#);

        body.push_str(&wrapped);
    }

    let html = layout(
        &body,
        &context,
        Location::AuxiliaryPage,
        &session_state,
        &site,
        site.title()
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::Context;
use crate::editor::Location;
use crate::editor::widgets::layout;

pub async fn deployment(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();
    let site = context.get_site();

    let mut body = String::new();

    let t_deployment = language.translations.deployment;
    body.push_str(&format!("<h2>{t_deployment}</h2>"));

    // TODO: Localize here and below
    let target = match &site.base_url {
        Some(hyper_url) => {
            let url = &hyper_url.representation;
            format!("<p>Files are being deployed to <a href={url}>{url}</a></p>")
        }
        None => String::new()
    };

    let text = formatdoc!(r#"
        <p>This lists the output from the deployment background process, if there's any.</p>
        {target}
    "#);
    body.push_str(&text);

    if let Ok(deployment) = context.deploying.lock() {
        let feedback = &deployment.feedback;
        let wrapped = formatdoc!(r#"
            <pre id="deployment_feedback">
                {feedback}
            </pre>
        "#);

        body.push_str(&wrapped);
    }

    let html = layout(
        &body,
        &context,
        &language,
        Location::AuxiliaryPage,
        &site,
        site.title()
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

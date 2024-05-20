use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::Context;
use crate::editor::widgets::layout;

pub async fn deployment(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();
    let site = context.get_site();

    let mut body = String::new();

    let t_deployment = language.translations.deployment;
    body.push_str(&format!("<h2>{t_deployment}</h2>"));
    // TODO: Localize
    body.push_str(r#"<p>This lists the output from the deployment background process, if there's any.</p>"#);

    if let Ok(deployment) = context.deploying.lock() {
        let feedback = &deployment.feedback;
        let wrapped = formatdoc!(r#"
            <pre id="deployment_feedback">
                {feedback}
            </pre>
        "#);

        body.push_str(&wrapped);
    }

    let html = layout(&context, &language, None, &site, &body, site.title());

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::Context;
use crate::editor::widgets::layout;
use crate::util::precise_duration;

pub async fn jobs(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();

    let site = context.get_site();

    let mut body = String::new();

    // TODO: Localize
    body.push_str(r#"<h2>Background processing dashboard</h2>"#);
    body.push_str(r#"<p>This lists active/failed/pending/complete background jobs, if any, in this session</p>"#);

    if let Ok(processing_errors) = context.processing_errors.lock() {
        if !processing_errors.is_empty() {
            let errors = processing_errors
                .iter()
                .map(|err| format!("- {err}<br>"))
                .collect::<Vec<String>>()
                .join("\n");

            let wrapped = formatdoc!(r#"
                <div>
                    Processing errors:<br>
                    {errors}
                </div>
            "#);

            body.push_str(&wrapped);
        }
    }

    body.push_str(r#"<ul style="display: flex; flex-direction: column; list-style: none; padding-left: 0; row-gap: .5rem;">"#);

    let pending_jobs = context.job_queue
        .lock()
        .unwrap()
        .iter()
        .map(|job| {
            let label = job.kind.representation(&language.translations);
            // TODO: Localize
            let status = match job.assigned_at {
                Some(assigned_at) => format!("Started {:?}", assigned_at),
                None => "Not yet started".to_string()
            };

            formatdoc!(r#"
                <li>
                    {label}<br>
                    <small>{status}</small>
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    body.push_str(&pending_jobs);

    let completed_jobs = context.jobs_completed
        .lock()
        .unwrap()
        .iter()
        .map(|job| {
            let label = job.kind.representation(&language.translations);
            // TODO: Localize
            let status = match job.duration() {
                Some(duration) => format!("Completed in {}", precise_duration(duration.as_secs_f32())),
                None => String::new()
            };

            formatdoc!(r#"
                <li>
                    {label}<br>
                    <small>{status}</small>
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    body.push_str(&completed_jobs);

    body.push_str("</ul>");

    let html = layout(&context, &language, None, &site, &body, site.title());

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

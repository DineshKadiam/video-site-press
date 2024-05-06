use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use chrono::DateTime;
use chrono::offset::Local;
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

    if let Ok(processing) = context.processing.lock() {
        if !processing.operations.is_empty() {
            let operations = processing.operations
                .iter()
                .map(|operation| {
                    let label = &operation.label;
                    let last_progress = match &operation.last_progress {
                        Some(progress) => format!("(Frame {})", progress.frame),
                        None => String::from("(No progress information yet)")
                    };
                    format!("<li>{label} {last_progress}</li>")
                })
                .collect::<Vec<String>>()
                .join("\n");

            let wrapped = formatdoc!(r#"
                <div>
                    Active processing:
                    <ul>
                        {operations}
                    </ul>
                </div>
            "#);

            body.push_str(&wrapped);
        }
    }

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

    let mut jobs_sorted = Vec::new();

    for job in &*context.job_queue.lock().unwrap() {
        let label = job.kind.representation(&language.translations);
        // TODO: Localize
        let status = match job.assigned_at {
            Some(assigned_at) => {
                let datetime: DateTime<Local> = assigned_at.into();
                format!("Started {}", datetime.format("%T"))
            }
            None => {
                let datetime: DateTime<Local> = job.created.into();
                format!("Created {}", datetime.format("%T"))
            }
        };

        let rendered = formatdoc!(r#"
            <li>
                {label}<br>
                <small>{status}</small>
            </li>
        "#);

        jobs_sorted.push((job.created, rendered));
    }

    for job in &*context.jobs_completed.lock().unwrap() {
        let label = job.kind.representation(&language.translations);
        // TODO: Localize
        let status = match job.duration() {
            Some(duration) => format!("Completed in {}", precise_duration(duration.as_secs_f32())),
            None => String::new()
        };

        let rendered = formatdoc!(r#"
            <li>
                {label}<br>
                <small>{status}</small>
            </li>
        "#);

        jobs_sorted.push((job.created, rendered));
    }

    jobs_sorted.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    let jobs_rendered = jobs_sorted
        .into_iter()
        .map(|created_rendered| created_rendered.1)
        .collect::<Vec<String>>()
        .join("\n");

    body.push_str(&jobs_rendered);

    body.push_str("</ul>");

    let html = layout(&context, &language, None, &site, &body, site.title());

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

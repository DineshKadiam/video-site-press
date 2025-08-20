// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use chrono::DateTime;
use chrono::offset::Local;
use indoc::formatdoc;

use crate::Context;
use crate::editor::Location;
use crate::editor::widgets::layout;
use crate::util::precise_duration;

pub async fn jobs(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();

    let site = context.get_site();

    let mut body = String::new();

    let t_background_jobs = language.translations.background_jobs;
    body.push_str(&format!("<h2>{t_background_jobs}</h2>"));

    // TODO: Localize
    body.push_str(r#"<p>This lists active/failed/pending/complete background jobs, if any, in this session</p>"#);

    if let Ok(active_jobs) = context.active_jobs.lock() {
        if !active_jobs.jobs.is_empty() {
            let jobs = active_jobs.jobs
                .iter()
                .map(|job| {
                    let label = &job.label;
                    let last_progress = match &job.last_progress {
                        Some(progress) => format!("({}%)", progress.percent),
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
                        {jobs}
                    </ul>
                </div>
            "#);

            body.push_str(&wrapped);
        }
    }

    if let Ok(job_errors) = context.job_errors.lock() {
        if !job_errors.is_empty() {
            let errors = job_errors
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
        // TODO: Html-escape user content (title), generally across this file (jobs.rs)
        let label = job.kind.label(&language.translations);
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

    for job in &*context.completed_jobs.lock().unwrap() {
        let label = job.kind.label(&language.translations);
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

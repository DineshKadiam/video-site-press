// SPDX-FileCopyrightText: 2024-2025 Simon Repp
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
use crate::util::{
    html_escape_outside_attribute,
    precise_duration
};

pub async fn jobs(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();
    let translations = &language.translations;

    let site = context.get_site();

    let mut body = String::new();

    let t_background_jobs = translations.background_jobs;
    body.push_str(&format!("<h2>{t_background_jobs}</h2>"));

    let mut no_jobs = true;

    let mut jobs_sorted = Vec::new();

    for job in &*context.job_queue.lock().unwrap() {
        let label = job.desc.label(&translations);
        let e_label = html_escape_outside_attribute(&label);

        // TODO: Localize

        let created_at_datetime: DateTime<Local> = job.created_at.into();
        let created_at_formatted = created_at_datetime.format("%T");
        let created_at = format!("Created {created_at_formatted}");

        let status = if let Some(assignment) = &job.active_assignment {
            let started_at_datetime: DateTime<Local> = assignment.started_at.into();
            let started_at_formatted = started_at_datetime.format("%T");
            let worker_number = assignment.worker_number;
            format!(r#" &gt; <strong style="color: yellow;">Started</strong> {started_at_formatted} by worker {worker_number}"#)
        } else if let Some(failure) = &job.failure {
            let error = &failure.error;
            let duration = failure.duration();
            let duration_formatted = precise_duration(duration.as_secs_f32());
            format!(r#" &gt; <strong style="color: red;">Failed</strong> after {duration_formatted} ({error})"#)
        } else {
            String::new()
        };

        let rendered = formatdoc!(r#"
            <li>
                {e_label}<br>
                <small>{created_at}{status}</small>
            </li>
        "#);

        jobs_sorted.push((job.created_at, rendered));
    }

    for job in &*context.completed_jobs.lock().unwrap() {
        let label = job.desc.label(&translations);
        let e_label = html_escape_outside_attribute(&label);

        // TODO: Localize

        let duration = job.duration();
        let duration_formatted = precise_duration(duration.as_secs_f32());
        let status = format!("Completed in {duration_formatted}");

        let rendered = formatdoc!(r#"
            <li>
                {e_label}<br>
                <small>{status}</small>
            </li>
        "#);

        jobs_sorted.push((job.created_at, rendered));
    }

    if !jobs_sorted.is_empty() {
        jobs_sorted.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        let jobs_rendered = jobs_sorted
            .into_iter()
            .map(|created_rendered| created_rendered.1)
            .collect::<Vec<String>>()
            .join("\n");

        let jobs_container = formatdoc!(r#"
            <ul class="jobs">
                {jobs_rendered}
            </ul>
        "#);

        body.push_str(&jobs_container);

        no_jobs = false;
    }

    if no_jobs {
        let t_no_jobs_yet = translations.no_jobs_yet;
        let no_jobs_message = format!("<div>{t_no_jobs_yet}</div>");
        body.push_str(&no_jobs_message);
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

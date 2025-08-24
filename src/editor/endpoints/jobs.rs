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
    casual_duration,
    html_escape_outside_attribute
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
        let output_label = job.desc.output_label(&translations);
        let target_label = job.desc.target_label(&translations);
        let target_label_escaped = html_escape_outside_attribute(&target_label);
        let target_edit_url = job.desc.target_edit_url();
        let target_link = format!(r#"<a href="{target_edit_url}">{target_label_escaped}</a>"#);


        let mut error = String::new();
        let status = if let Some(assignment) = &job.active_assignment {
            let started_at_datetime: DateTime<Local> = assignment.started_at.into();
            let started_at_formatted = started_at_datetime.format("%T");
            let t_processing = translations.processing;
            let worker_number = assignment.worker_number;
            format!(r#"<strong style="color: yellow;">{t_processing}</strong> {started_at_formatted} (#{worker_number})"#)
        } else if let Some(failure) = &job.failure {
            error = format!("<div>{}</div>", failure.error);
            let duration = casual_duration(failure.duration().as_secs_f32(), translations);
            let t_failed = translations.failed;
            format!(r#"<strong style="color: red;">{t_failed}</strong> ({duration})"#)

        } else {
            let created_at_datetime: DateTime<Local> = job.created_at.into();
            let created_at_formatted = created_at_datetime.format("%T");
            let t_queued = translations.queued;
            format!("{t_queued} ({created_at_formatted})")
        };

        let rendered = formatdoc!(r#"
            <li>
                <div>
                    {target_link}
                    <div>{output_label}</div>
                    {error}
                </div>
                <small>{status}</small>
            </li>
        "#);

        jobs_sorted.push((job.created_at, rendered));
    }

    for job in &*context.completed_jobs.lock().unwrap() {
        let output_label = job.desc.output_label(&translations);
        let target_label = job.desc.target_label(&translations);
        let target_label_escaped = html_escape_outside_attribute(&target_label);
        let target_edit_url = job.desc.target_edit_url();
        let target_link = format!(r#"<a href="{target_edit_url}">{target_label_escaped}</a>"#);

        let duration = casual_duration(job.duration().as_secs_f32(), translations);
        let t_completed = translations.completed;
        let status = format!(r#"<strong style="color: green;">{t_completed}</strong> ({duration})"#);

        let rendered = formatdoc!(r#"
            <li>
                <div>
                    {target_link}
                    <div>{output_label}</div>
                </div>
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

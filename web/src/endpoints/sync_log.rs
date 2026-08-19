// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use chrono::DateTime;
use chrono::offset::Local;
use indoc::formatdoc;

use hyper8_core::{SyncEventActionKind, SyncEventKind};
use hyper8_util::{html_escape_outside_attribute, human_size_delta};

use crate::{
    Context,
    Layout,
    Location,
    SessionState
};

/// Page to follow the sync progress in real time (updated via client scripting)
pub async fn sync_log(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let site = context.get_site();

    let translations = &session_state.language.translations;

    let mut body = String::new();

    let t_sync = translations.sync;
    body.push_str(&format!("<h2>{t_sync}</h2>"));

    if let Some(hyper_url) = &site.base_url {
        let url = &hyper_url.representation;
        let link = format!(r#"<a href="{url}" target="_blank">{url}</a>"#);
        let t_files_are_being_synchronized_to_xxx = translations
            .files_are_being_synchronized_to_xxx(&link);

        let text = format!("<p>{t_files_are_being_synchronized_to_xxx}</p>");
        body.push_str(&text);
    }

    if let Some(sync_state) = &*context.sync_state.lock().unwrap() {
        let simulation = if sync_state.simulation {
            let t_simulation = translations.simulation;
            format!(r#"<span class="simulation">{t_simulation}</span>"#)
        } else {
            String::new()
        };

        let log = sync_state.log
            .iter()
            .map(|event| {
                let progress = if let Some(step) = event.step {
                    let steps = sync_state.steps.unwrap();
                    format!(r#"<span class="progress">({step}/{steps})</span> "#)
                } else {
                    String::new()
                };

                let message = match &event.kind {
                    SyncEventKind::Action { kind, path } => {
                        let kind = match kind {
                            SyncEventActionKind::AddFile { size_delta } => {
                                let size_formatted = human_size_delta(*size_delta);
                                let t_add_file = translations.add_file;
                                format!(r#"<span class="action add_file">{t_add_file}</span><span class="size">{size_formatted}</span>"#)
                            }
                            SyncEventActionKind::CreateDir => {
                                let t_create_directory = translations.create_directory;
                                format!(r#"<span class="action create_dir">{t_create_directory}</span>"#)
                            }
                            SyncEventActionKind::DeleteDir => {
                                let t_delete_directory = translations.delete_directory;
                                format!(r#"<span class="action delete_dir">{t_delete_directory}</span>"#)
                            }
                            SyncEventActionKind::DeleteFile { size_delta } => {
                                let size_formatted = human_size_delta(*size_delta);
                                let t_delete_file = translations.delete_file;
                                format!(r#"<span class="action delete_file">{t_delete_file}</span><span class="size">{size_formatted}</span>"#)
                            }
                            SyncEventActionKind::UpdateFile { size_delta } => {
                                let size_formatted = human_size_delta(*size_delta);
                                let t_update_file = translations.update_file;
                                format!(r#"<span class="action update_file">{t_update_file}</span><span class="size">{size_formatted}</span>"#)
                            }
                        };

                        let path_escaped = html_escape_outside_attribute(path);

                        format!(r#"{kind}<span class="path">{path_escaped}</span>"#)
                    }
                    SyncEventKind::Error(message) => {
                        let message_escaped = html_escape_outside_attribute(message);
                        format!(r#"<span class="err">{message_escaped}</span>"#)
                    }
                    SyncEventKind::Generic(message) => {
                        let message_escaped = html_escape_outside_attribute(message);
                        format!(r#"<span class="message">{message_escaped}</span>"#)
                    }
                };

                let started_at_datetime: DateTime<Local> = event.started_at.into();
                let started_at_formatted = started_at_datetime.format("%T");

                formatdoc!("
                    <li>{simulation}<span>[{started_at_formatted}]</span>{progress}{message}</li>
                ")
            })
            .collect::<Vec<_>>()
            .join("");

        let events_rendered = sync_state.log.len();
        let session_id = &sync_state.session_id;

        let wrapped = formatdoc!(r#"
            <ul data-events-rendered="{events_rendered}" data-session-id="{session_id}" id="sync_log">
                {log}
            </ul>
        "#);

        body.push_str(&wrapped);
    }

    let mut layout = Layout::new();

    layout.wide();

    let html = layout.render(
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

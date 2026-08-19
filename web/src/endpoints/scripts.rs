// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, SessionState};

pub async fn scripts(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let static_js = include_str!("../assets/editor.js");

    let session_language = SessionState::language(&context.server_mode, &session);
    let translations = session_language.translations;

    // We selectively provide only those translations to the runtime js code
    // that are absolutely required, handpicking them here.
    let t_add_file = translations.add_file;
    let t_create_directory = translations.create_directory;
    let t_delete_directory = translations.delete_directory;
    let t_delete_file = translations.delete_file;
    let t_idle = translations.idle;
    let t_new_results_available = translations.new_results_available;
    let t_really_sync = translations.really_sync;
    let t_refresh = translations.refresh;
    let t_simulation = translations.simulation;
    let t_sync_finished = translations.sync_finished;
    let t_syncing = translations.syncing;
    let t_update_file = translations.update_file;
    let t_xxx_active_jobs = translations.xxx_active_jobs;
    let t_xxx_pending_jobs = translations.xxx_pending_jobs;

    let js = formatdoc!(r#"
        const t = {{
            addFile: '{t_add_file}',
            createDirectory: '{t_create_directory}',
            deleteDirectory: '{t_delete_directory}',
            deleteFile: '{t_delete_file}',
            idle: '{t_idle}',
            newResultsAvailable: '{t_new_results_available}',
            reallySync: '{t_really_sync}',
            refresh: '{t_refresh}',
            simulation: '{t_simulation}',
            syncFinished: '{t_sync_finished}',
            syncing: '{t_syncing}',
            updateFile: '{t_update_file}',
            xxxActiveJobs: count => '{t_xxx_active_jobs}'.replace('{{count}}', count),
            xxxPendingJobs: count => '{t_xxx_pending_jobs}'.replace('{{count}}', count)
        }};
        {static_js}
    "#);

    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(js)
}

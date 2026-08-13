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
    let t_deploying = translations.deploying;
    let t_deployment_finished = translations.deployment_finished;
    let t_idle = translations.idle;
    let t_new_results_available = translations.new_results_available;
    let t_really_deploy = translations.really_deploy;
    let t_refresh = translations.refresh;
    let t_xxx_active_jobs = translations.xxx_active_jobs;
    let t_xxx_pending_jobs = translations.xxx_pending_jobs;

    let js = formatdoc!(r#"
        const t = {{
            deploying: '{t_deploying}',
            deploymentFinished: '{t_deployment_finished}',
            idle: '{t_idle}',
            newResultsAvailable: '{t_new_results_available}',
            reallyDeploy: '{t_really_deploy}',
            refresh: '{t_refresh}',
            xxxActiveJobs: count => '{t_xxx_active_jobs}'.replace('{{count}}', count),
            xxxPendingJobs: count => '{t_xxx_pending_jobs}'.replace('{{count}}', count)
        }};
        {static_js}
    "#);

    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(js)
}

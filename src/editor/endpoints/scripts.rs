// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::Context;

pub async fn scripts(context: Data<Arc<Context>>) -> HttpResponse {
    let static_js = include_str!("../assets/scripts.js");

    let translations = context.get_editor_language().translations;

    // We selectively provide only those translations to the runtime js code
    // that are absolutely required, handpicking them here.
    let t_configure_deployment = translations.configure_deployment;
    let t_deploy = translations.deploy;
    let t_deploying = translations.deploying;
    let t_idle = translations.idle;
    let t_really_deploy = translations.really_deploy;
    let translations_js = formatdoc!(r#"
        const t = {{
            configureDeployment: '{t_configure_deployment}',
            deploy: '{t_deploy}',
            deploying: '{t_deploying}',
            idle: '{t_idle}',
            reallyDeploy: '{t_really_deploy}'
        }};
    "#);

    let js = formatdoc!(r#"
        {translations_js}
        {static_js}
    "#);

    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(js)
}

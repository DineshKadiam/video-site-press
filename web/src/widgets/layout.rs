// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_core::{Site, SitePath};
use hyper8_core::util::{
    html_escape_outside_attribute,
    human_size,
    precise_duration
};
use hyper8_translations::Translations;
use hyper8_version::VERSION_WITH_PATCH;

use crate::{Context, Location, SessionState};
use crate::routes;

use super::{splash_screen, tree_view};

fn controls(
    context: &Context,
    site: &Site,
    translations: &Translations
) -> String {
    let t_configure_deployment = translations.configure_deployment;
    let configure_icon = hyper8_icons::configure(Some(translations.configure));
    let deployment_icon = hyper8_icons::deployment(translations.deployment);
    let deployment_button = match site.complete_deployment_config() {
        true => {
            let deployment = &context.deploying.lock().unwrap();
            if deployment.active() {
                let t_deploying  = translations.deploying;
                let duration = precise_duration(deployment.started_at.unwrap().elapsed().map(|duration| duration.as_secs_f32()).unwrap_or(0.0));
                formatdoc!(r#"
                    <a href="/deployment">
                        {deployment_icon} {t_deploying} ({duration})
                    </a>
                "#)
            } else {
                let base_url = match &site.base_url {
                    Some(base_url) => format!(" ({})", base_url.representation),
                    None => String::new()
                };
                let t_deploy = translations.deploy;
                let t_really_deploy = translations.really_deploy;

                let deploy_action = routes::DEPLOY;
                formatdoc!(r#"
                    <form
                        action="{deploy_action}"
                        data-confirm
                        data-confirm-label="{t_deploy}"
                        data-confirm-text="{t_really_deploy}{base_url}"
                        method="post"">
                        <button>{deployment_icon} {t_deploy}</button>
                    </form>
                "#)
            }
        }
        false => {
            let site_edit_url = routes::SITE_EDIT;
            formatdoc!(r#"
                <a href="{site_edit_url}">
                    {configure_icon} {t_configure_deployment}
                </a>
            "#)
        }
    };
    let deployment = format!(r#"<div id="deployment">{deployment_button}</div>"#);

    let t_processing = translations.processing;
    let processing_icon = hyper8_icons::processing(t_processing);
    let processing = {
        let job_queue = &context.job_queue.lock().unwrap();

        if job_queue.is_empty() {
            let t_idle = translations.idle;
            format!(r#"<a class="processing" href="/jobs">{processing_icon} {t_idle}</a>"#)
        } else {
            let mut failed_jobs = 0;
            let mut pending_jobs = 0;

            let jobs = job_queue
                .iter()
                .filter_map(|job| {
                    if let Some(assignment) = &job.active_assignment {
                        let badge = match &assignment.last_progress {
                            Some(last_progress) => {
                                let progress_percent = last_progress.percent;
                                let time_remaining = precise_duration(last_progress.duration_remaining);

                                formatdoc!(r#"
                                    <div class="processing_badge">
                                        <span>{progress_percent}%</span>
                                        <span class="inverted">-{time_remaining}</span>
                                    </div>
                                "#)
                            }
                            None => {
                                formatdoc!(r#"
                                    <div class="processing_badge">
                                        MISC
                                    </div>
                                "#)
                            }
                        };

                        Some(badge)
                    } else {
                        if job.failure.is_some() {
                            failed_jobs += 1;
                        } else {
                            pending_jobs += 1;
                        }

                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("");

            let failed_badge = if failed_jobs > 0 {
                format!(" <span>⚠{failed_jobs}</span>")
            } else {
                String::new()
            };

            let pending_badge = if pending_jobs > 0 {
                format!(" <span>+{pending_jobs}</span>")
            } else {
                String::new()
            };

            formatdoc!(r#"
                <a class="active processing" href="/jobs">
                    {processing_icon} {jobs}{failed_badge}{pending_badge}
                </a>
            "#)
        }
    };

    let t_preview = translations.preview;
    let preview_icon = hyper8_icons::preview(t_preview);
    let preview_url = routes::preview(&SitePath::root_path());
    let preview = formatdoc!(r#"
        <a href="{preview_url}" target="_blank">
            {preview_icon} {t_preview}
        </a>
        <script>
            (()=>{{
                const previewLink = document.currentScript.previousElementSibling;

                // Work around https://github.com/tauri-apps/plugins-workspace/issues/2990
                // TODO: Remove/revisit as soon as we have an official solution
                if (window.__TAURI__) {{
                    const {{ openUrl }} = window.__TAURI__.opener;
                    previewLink.addEventListener('click', event => {{
                        event.preventDefault();
                        openUrl(previewLink.href);
                    }})
                }} else {{
                    // When running the editor inside the desktop application, this link can only be opened
                    // if it points to an absolute url, hence we transform the relative to an absolute url here.
                    // Note that although the href attribute is relative here (e.g. "/build/"), accessing .href
                    // returns the fully resolved url, therefore we replace the relative url with a fully resolved
                    // one here.
                    previewLink.setAttribute('href', previewLink.href);
                }}
            }})();
        </script>
    "#);

    let storage_icon = hyper8_icons::storage(Some(translations.storage_usage));
    let storage_usage = human_size(site.storage_usage());
    let storage = format!(r#"<a href="/">{storage_icon} {storage_usage}</a>"#);

    let logo_polychrome_icon = hyper8_icons::logo_polychrome(Some("Hyper 8"));
    let t_about = translations.about;
    let about = format!(r#"<button id="toggle-splash">{logo_polychrome_icon} {t_about}</button>"#);

    formatdoc!(r#"
        <div class="controls">
            {processing}
            {storage}
            {preview}
            {deployment}
            {about}
            <template id="configure_icon">
                {configure_icon}
            </template>
            <template id="deployment_icon">
                {deployment_icon}
            </template>
            <template id="processing_icon">
                {processing_icon}
            </template>
        </div>
    "#)
}

pub fn layout(
    body: &str,
    context: &Context,
    location: Location,
    session_state: &SessionState,
    site: &Site,
    title: &str
) -> String {
    let translations = &session_state.language.translations;
    let (breadcrumbs, tree) = tree_view(
        location,
        session_state,
        site,
        &context.site_dir
    );

    // TODO: Hash scripts/styles
    let e_title = html_escape_outside_attribute(title);
    let head = formatdoc!(r#"
        <head>
            <title>{e_title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="/favicon.png" rel="icon" type="image/png">
            <link rel="stylesheet" href="/editor.css">
            <script defer src="/editor.js" type="module"></script>
        </head>
    "#);


    let splash_screen_rendered = splash_screen(session_state);

    let r_controls = controls(context, site, translations);

    let tree_view_order_label = session_state.tree_view_order.label(translations);

    // This is used at runtime in client-side js to display the url in a message
    let data_base_url = match &site.base_url {
        Some(base_url) => format!(r#"data-base-url="{}""#, base_url.representation),
        None => String::new()
    };
    // We put this on body as the data-version attribute, and this is
    // subsequently used at runtime to determine whether we need to display
    // the splash screen (again), if it hasn't yet been displayed for the
    // given version.
    let t_cancel = translations.cancel;
    let t_filter = translations.filter;
    let body = formatdoc!(r#"
        <body {data_base_url} data-version="{VERSION_WITH_PATCH}">
            <div class="split">
                <nav>
                    <div class="tree_view_options">
                        <input id="filter_tree" placeholder="{t_filter}" type="search">
                        <form action="/change-tree-view-order" id="tree-view-order" method="post">
                            <input name="redirect" type="hidden">
                            <button>{tree_view_order_label} ↓</button>
                        </form>
                    </div>
                    {tree}
                </nav>
                <div>
                    {r_controls}
                    {breadcrumbs}
                    <main>
                        {body}
                    </main>
                </div>
            </div>
            {splash_screen_rendered}
            <template id="confirmation_dialog">
                <dialog class="confirmation_wrapper">
                    <div class="text"></div>
                    <div class="buttons">
                        <button autofocus class="cancel">{t_cancel}</button>
                        <button class="confirm danger"></button>
                    </div>
                </dialog>
            </template>
        </body>
    "#);

    formatdoc!(r#"
        <!doctype html>
        <html lang="en">
            {head}
            {body}
        </html>
    "#)
}

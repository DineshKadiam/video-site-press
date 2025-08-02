// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{
    Context,
    Language,
    Site,
    Translations
};
use crate::editor::Location;
use crate::editor::routes;
use crate::icons;
use crate::util::{
    html_escape_outside_attribute,
    human_size,
    precise_duration
};

use super::{splash_screen, tree_view};

fn controls(
    context: &Context,
    site: &Site,
    translations: &Translations
) -> String {
    let t_configure_deployment = translations.configure_deployment;
    let configure_icon = icons::configure(Some(translations.configure));
    let deployment_icon = icons::deployment(translations.deployment);
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
    let processing_icon = icons::processing(t_processing);
    let processing = {
        let processing = &context.processing.lock().unwrap();
        if processing.operations.is_empty() {
            let t_idle = translations.idle;
            format!(r#"<a class="processing" href="/jobs">{processing_icon} {t_idle}</a>"#)
        } else {
            let operations = processing.operations
                .iter()
                .map(|operation| {
                    match &operation.last_progress {
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
                    }
                })
                .collect::<Vec<String>>()
                .join("");

            formatdoc!(r#"
                <a class="active processing" href="/jobs">
                    {processing_icon} {operations}
                </a>
            "#)
        }
    };

    let t_preview = translations.preview;
    let preview_icon = icons::preview(t_preview);
    let preview = formatdoc!(r#"
        <form action="/build/" method="post" target="_blank">
            <button>{preview_icon} {t_preview}</button>
        </form>
    "#);

    let storage_icon = icons::storage(Some(translations.storage_usage));
    let storage_usage = human_size(site.storage_usage());
    let storage = format!(r#"<a href="/">{storage_icon} {storage_usage}</a>"#);

    let logo_polychrome_icon = icons::logo_polychrome(Some("Hyper 8"));
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
    language: &Language,
    location: Location,
    site: &Site,
    title: &str
) -> String {
    let translations = &language.translations;
    let (breadcrumbs, tree) = tree_view(
        context,
        location,
        site,
        &context.site_dir,
        translations
    );

    // TODO: Hash scripts/styles
    let e_title = html_escape_outside_attribute(title);
    let head = formatdoc!(r#"
        <head>
            <title>{e_title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="/favicon.png" rel="icon" type="image/png">
            <!-- <link href="favicon_light.png" rel="icon" type="image/png" media="(prefers-color-scheme: light)"> -->
            <script defer src="/scripts.js" type="module"></script>
            <link rel="stylesheet" href="/styles.css">
        </head>
    "#);

    let splash_screen_rendered = splash_screen(language, context.get_editor_theme());

    let r_controls = controls(context, site, translations);

    let tree_view_order_label = context.get_editor_tree_view_order().key(translations);

    // This is used at runtime in client-side js to display the url in a message
    let data_base_url = match &site.base_url {
        Some(base_url) => format!(r#"data-base-url="{}""#, base_url.representation),
        None => String::new()
    };
    // We put this on body as the data-version attribute, and this is
    // subsequently used at runtime to determine whether we need to display
    // the splash screen (again), if it hasn't yet been displayed for the
    // given version.
    let version_detailed = env!("HYPER8_VERSION_DETAILED");
    let t_cancel = translations.cancel;
    let t_filter = translations.filter;
    let body = formatdoc!(r#"
        <body {data_base_url} data-version="{version_detailed}">
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

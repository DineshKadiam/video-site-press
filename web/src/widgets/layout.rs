// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-FileCopyrightText: 2024 Walter Ebert
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_core::{Site, SitePath};
use hyper8_translations::Translations;
use hyper8_util::{
    html_escape_outside_attribute,
    human_size,
    precise_duration
};
use hyper8_version::{VERSION_ADAPTIVE_PATCH, VERSION_WITH_PATCH};

use crate::{
    Context,
    EditorTheme,
    Location,
    SessionState
};
use crate::routes;

use super::{splash_screen, tree_view};

pub struct Layout {
    column: bool
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            column: true
        }
    }

    pub fn render(
        &self,
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

        let r_footer = footer(session_state, translations);

        let change_tree_view_order_url = routes::CHANGE_TREE_VIEW_ORDER;

        let class_column = if self.column { r#"class="column""# } else { "" };

        let t_cancel = translations.cancel;
        let t_filter = translations.filter;
        formatdoc!(r#"
            <!doctype html>
            <html lang="en">
                {head}
                <body {data_base_url} data-version="{VERSION_WITH_PATCH}">
                    <div class="split">
                        <nav>
                            <div class="tree_view_options">
                                <input id="filter_tree" placeholder="{t_filter}" type="search">
                                <form action="{change_tree_view_order_url}" id="tree-view-order" method="post">
                                    <input name="redirect" type="hidden">
                                    <button>{tree_view_order_label} ↓</button>
                                </form>
                            </div>
                            {tree}
                        </nav>
                        <div>
                            {r_controls}
                            {breadcrumbs}
                            <main {class_column}>
                                {body}
                            </main>
                        </div>
                        {r_footer}
                    </div>
                    {splash_screen_rendered}
                    <template id="modal">
                        <dialog class="modal">
                            <div class="text"></div>
                            <div class="buttons">
                                <button class="cancel">{t_cancel}</button>
                                <button class="confirm danger"></button>
                            </div>
                        </dialog>
                    </template>
                </body>
            </html>
        "#)
    }

    pub fn wide(&mut self) {
        self.column = false;
    }
}

fn controls(
    context: &Context,
    site: &Site,
    translations: &Translations
) -> String {
    let sync_status = {
        if let Some(sync_state) = &*context.sync_state.lock().unwrap() {
            let duration = precise_duration(sync_state.duration_secs());

            let label = if sync_state.finished() {
                translations.sync_finished
            } else {
                translations.syncing
            };

            formatdoc!(r#"
                <a href="/sync_log">
                    {label} ({duration})
                </a>
            "#)
        } else {
            String::new()
        }
    };

    let ftp_sync = if site.ftp_config.complete() {
        let t_configuration_check = translations.configuration_check;
        let t_simulate_sync = translations.simulate_sync;
        let t_sync = translations.sync;

        let sync_check_ftp_action = routes::SYNC_CHECK_FTP;
        let sync_ftp_action = routes::SYNC_FTP;
        let simulate_sync_ftp_action = routes::SIMULATE_SYNC_FTP;
        let ftp_ftps = if site.ftp_config.secure() { "FTPS" } else { "FTP" };
        let server = site.ftp_config.server_unchecked();
         formatdoc!(r#"
            <li>
                <div class="section">{ftp_ftps} <span>{server}</span></div>
                <ul>
                    <li>
                        <form
                            action="{sync_ftp_action}"
                            data-sync-confirm
                            data-sync-precheck
                            data-sync-protocol="ftp"
                            method="post">
                            <button>{t_sync}</button>
                        </form>
                    </li>
                    <li>
                        <form
                            action="{simulate_sync_ftp_action}"
                            data-sync-precheck
                            data-sync-protocol="ftp"
                            method="post">
                            <button>{t_simulate_sync}</button>
                        </form>
                    </li>
                    <li>
                        <form
                            action="{sync_check_ftp_action}"
                            data-sync-precheck
                            data-sync-protocol="ftp"
                            method="post">
                            <button>{t_configuration_check}</button>
                        </form>
                    </li>
                </ul>
            </li>
        "#)
    } else {
        String::new()
    };

    let ssh_sync = if site.ssh_config.complete() {
        let t_configuration_check = translations.configuration_check;
        let t_simulate_sync = translations.simulate_sync;
        let t_sync = translations.sync;

        let sync_check_ssh_action = routes::SYNC_CHECK_SSH;
        let sync_ssh_action = routes::SYNC_SSH;
        let simulate_sync_ssh_action = routes::SIMULATE_SYNC_SSH;
        let server = site.ssh_config.server_unchecked();
        formatdoc!(r#"
            <li>
                <div class="section">SSH <span>{server}</span></div>
                <ul>
                    <li>
                        <form
                            action="{sync_ssh_action}"
                            data-sync-confirm
                            data-sync-precheck
                            data-sync-protocol="ssh"
                            method="post">
                            <button>{t_sync}</button>
                        </form>
                    </li>
                    <li>
                        <form
                            action="{simulate_sync_ssh_action}"
                            data-sync-precheck
                            data-sync-protocol="ssh"
                            method="post">
                            <button>{t_simulate_sync}</button>
                        </form>
                    </li>
                    <li>
                        <form
                            action="{sync_check_ssh_action}"
                            data-sync-precheck
                            data-sync-protocol="ssh"
                            method="post">
                            <button>{t_configuration_check}</button>
                        </form>
                    </li>
                </ul>
            </li>
        "#)
    } else {
        String::new()
    };

    let sync_icon = hyper8_icons::SYNC;
    let site_edit_url = routes::SITE_EDIT;
    let t_edit_configuration = translations.edit_configuration;
    let t_sync = translations.sync;
    let sync_menu = formatdoc!(r#"
        <div class="menu" id="sync">
            <button>{sync_icon} {t_sync}</button>
            <div>
                <ul>
                    {ssh_sync}
                    {ftp_sync}
                    <li>
                        <a href="{site_edit_url}">
                            {t_edit_configuration}
                        </a>
                    </li>
                    <li class="sync_status">{sync_status}</li>
                </ul>
            </div>
        </div>
    "#);

    let t_processing = translations.processing;
    let processing_icon = hyper8_icons::processing(t_processing);
    let processing = {
        let job_queue = &context.job_queue.lock().unwrap();

        let jobs_url = routes::JOBS;

        if job_queue.is_empty() {
            let t_idle = translations.idle;
            format!(r#"<a class="processing" href="{jobs_url}">{processing_icon} {t_idle}</a>"#)
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
                <a class="active processing" href="{jobs_url}">
                    {processing_icon} {jobs}{failed_badge}{pending_badge}
                </a>
            "#)
        }
    };

    let t_preview = translations.preview;
    let preview_icon = hyper8_icons::PREVIEW;
    let preview_url = routes::preview(&SitePath::root_path());
    let preview = formatdoc!(r#"
        <a href="{preview_url}" target="_blank">
            {preview_icon} {t_preview}
        </a>
        <script>
            (()=>{{
                const previewLink = document.currentScript.previousElementSibling;

                // When running the editor inside the desktop application, this link can only be opened
                // if it points to an absolute url, hence we transform the relative to an absolute url here.
                // Note that although the href attribute is relative here (e.g. "/build/"), accessing .href
                // returns the fully resolved url, therefore we replace the relative url with a fully resolved
                // one here.
                previewLink.setAttribute('href', previewLink.href);
            }})();
        </script>
    "#);

    let dashboard_url = routes::DASHBOARD;
    let storage_icon = hyper8_icons::storage(Some(translations.storage_usage));
    let storage_usage = human_size(site.build_storage_usage());
    let storage = format!(r#"<a href="{dashboard_url}">{storage_icon} {storage_usage}</a>"#);

    let reload_icon = hyper8_icons::reload(None, None);
    formatdoc!(r#"
        <div class="controls">
            {processing}
            {storage}
            {preview}
            {sync_menu}
            <template data-processing-icon>{processing_icon}</template>
            <template data-reload-icon>{reload_icon}</template>
        </div>
    "#)
}

fn footer(
    session_state: &SessionState,
    translations: &Translations
) -> String {
    let logo_monochrome_icon = hyper8_icons::logo_monochrome(Some("Hyper 8"));
    let splash_link = format!(r#"<button id="toggle-splash">{logo_monochrome_icon} {VERSION_ADAPTIVE_PATCH}</button>"#);

    let language_icon = hyper8_icons::LANGUAGE;
    let mut language_options = String::new();
    for (code, label) in [("de", "Deutsch"), ("en", "English")] {
        let selected = if code == session_state.language.code { "selected" } else { "" };
        let option = format!(r#"<option {selected} value="{code}">{label}</option>"#);
        language_options.push_str(&option);
    }

    let theme_icon = session_state.theme.icon();
    let mut theme_options = String::new();
    for (theme, label) in EditorTheme::variants(translations) {
        let key = theme.key();
        let selected = if key == session_state.theme.key() { "selected" } else { "" };
        let option = format!(r#"<option {selected} value="{key}">{label}</option>"#);
        theme_options.push_str(&option);
    }

    let change_language_url = routes::CHANGE_LANGUAGE;
    let change_theme_url = routes::CHANGE_THEME;

    let t_language = translations.language;
    let t_theme = translations.theme;
    formatdoc!(r#"
        <footer>
            <div>
                {language_icon}
                <form action="{change_language_url}" method="post">
                    <input name="redirect" type="hidden">
                    <div class="form_split even">
                        <select aria-label="{t_language}" id="language_code" name="language_code">
                            {language_options}
                        </select>
                    </div>
                </form>
            </div>
            <div>
                {theme_icon}
                <form action="{change_theme_url}" method="post">
                    <input name="redirect" type="hidden">
                    <div class="form_split even">
                        <select aria-label="{t_theme}" id="theme_key" name="theme_key">
                            {theme_options}
                        </select>
                    </div>
                </form>
            </div>
            {splash_link}
        </footer>
    "#)
}

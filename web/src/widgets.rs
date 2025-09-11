// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::MAIN_SEPARATOR;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use hyper8_core::{Site,SitePath};
use hyper8_core::util::string_from_pathbuf;
use hyper8_translations::Translations;

use crate::{Context, Location, Session};
use crate::routes;

mod layout;
mod links;
mod splash_screen;
mod tree_view;

pub use layout::layout;
pub use links::{add_link_form, links};
use splash_screen::splash_screen;
use tree_view::tree_view;

pub fn error_page(
    context: &Context,
    message: &str,
    session: &Session
) -> HttpResponse {
    let t_back = session.language.translations.back;

    let dashboard_url = routes::DASHBOARD;
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="{dashboard_url}">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_error = session.language.translations.error;
    let html = layout(
        &body,
        context,
        Location::AuxiliaryPage,
        session,
        site,
        t_error
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn not_found(
    context: &Context,
    session: &Session,
    site_path: &SitePath
) -> HttpResponse {
    let translations = &session.language.translations;

    let t_back = translations.back;
    let t_message = translations.the_resource_xxx_was_not_found_message(site_path.normalized());

    let dashboard_url = routes::DASHBOARD;
    let body = formatdoc!(r#"
        <div>
            {t_message}
        </div>

        <a href="{dashboard_url}">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_resource_not_found = translations.resource_not_found;
    let html = layout(
        &body,
        context,
        Location::AuxiliaryPage,
        session,
        site,
        t_resource_not_found
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn resource_actions(
    context: &Context,
    offline: bool,
    site: &Site,
    site_path: &SitePath,
    translations: &Translations
) -> String {
    let local_path = string_from_pathbuf(context.site_dir.join(site_path.filesystem_relative()));
    let disk_icon = hyper8_icons::disk(translations.local_path);
    let local_path_displayed = if site_path.is_root_path() {
        format!("<span>{disk_icon} <span>{local_path}</span></span>")
    } else {
        let mut rsplitn = local_path.rsplitn(3, MAIN_SEPARATOR);
        rsplitn.next().unwrap();
        let slug = rsplitn.next().unwrap();
        let prefix = rsplitn.next().unwrap();
        format!(r#"<span>{disk_icon} <span>{prefix}{MAIN_SEPARATOR}<span class="path_extension">{slug}</span>{MAIN_SEPARATOR}</span></span>"#)
    };

    let copy_icon = hyper8_icons::COPY;

    let copy_live_url;
    let live_url_displayed;
    match &site.base_url {
        Some(base_url) => {
            let t_offline = translations.offline;
            let offline_label = if offline { format!(" ({t_offline})") } else { String::new() };

            let link_icon = hyper8_icons::link(Some(translations.external_link));
            let live_url = base_url.join_index(site, site_path);
            if site_path.is_root_path() {
                live_url_displayed = format!(r#"<a href="{live_url}" target="_blank">{link_icon} {live_url}{offline_label}</a>"#);
            } else {
                let mut rsplitn = live_url.rsplitn(3, '/');

                let index_html = rsplitn.next().unwrap();
                let slug = rsplitn.next().unwrap();
                let prefix = rsplitn.next().unwrap();

                live_url_displayed = format!(r#"<a href="{live_url}" target="_blank">{link_icon} {prefix}/<span class="path_extension">{slug}</span>/{index_html}{offline_label}</a>"#);
            };

            let t_copy_link = translations.copy_link;
            copy_live_url = formatdoc!(r#"
                <button class="action" data-copy="{live_url}">
                    <span class="icon">{copy_icon}</span>
                    {t_copy_link}
                </button>
            "#);
        }
        None => {
            copy_live_url = String::new();
            live_url_displayed = String::new();
        }
    };


    let preview_href;
    let preview_disabled;
    if offline {
        preview_disabled = "disabled";
        preview_href = String::from("");
    } else {
        let preview_url = routes::preview(site_path);

        preview_disabled = "";
        preview_href = format!(r#"href="{preview_url}""#)
    };

    let t_copy_path = translations.copy_path;
    let t_preview = translations.preview;
    let failure_icon = hyper8_icons::failure(translations.failed);
    let success_icon = hyper8_icons::success(translations.copied);
    let preview_icon = hyper8_icons::preview(t_preview);
    formatdoc!(r#"
        {live_url_displayed}
        {local_path_displayed}
        <div class="button_group">
            <a class="action {preview_disabled}" {preview_href} target="_blank">
                {preview_icon} {t_preview}
            </a>
            <script>
                (()=>{{
                    const previewLink = document.currentScript.previousElementSibling;
                    if (previewLink.href) {{
                        // When running the editor inside the desktop application, this link can only be opened
                        // if it points to an absolute url, hence we transform the relative to an absolute url here.
                        // Note that although the href attribute is relative here (e.g. "/build/"), accessing .href
                        // returns the fully resolved url, therefore we replace the relative url with a fully resolved
                        // one here.
                        previewLink.setAttribute('href', previewLink.href);
                    }}
                }})();
            </script>
            {copy_live_url}
            <button class="action" data-copy="{local_path}">
                <span class="icon">{copy_icon}</span>
                {t_copy_path}
            </button>
        </div>
        <template id="copy_icon">
            {copy_icon}
        </template>
        <template id="failure_icon">
            {failure_icon}
        </template>
        <template id="success_icon">
            {success_icon}
        </template>
    "#)
}

pub fn upload_widget(action: &str, label: &str) -> String {
    formatdoc!(r#"
        <form action="{action}" method="post" enctype="multipart/form-data">
            <input name="file" type="file">
            <button class="action upload_button">{label}</button>
        </form>
    "#)
}

// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{Context, Language, Site, Translations};
use crate::icons;

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
    language: &Language,
    message: &str
) -> HttpResponse {
    let t_back = language.translations.back;
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="/">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_error = language.translations.error;
    let html = layout(context, language, None, site, &body, t_error);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn not_found(
    context: &Context,
    language: &Language,
    path: &str
) -> HttpResponse {
    let t_back = language.translations.back;
    let t_message = language.translations.the_resource_xxx_was_not_found_message(path);
    let body = formatdoc!(r#"
        <div>
            {t_message}
        </div>

        <a href="/">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_resource_not_found = language.translations.resource_not_found;
    let html = layout(context, language, None, site, &body, t_resource_not_found);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn path_widget(context: &Context, path: &str, site: &Site, translations: &Translations) -> String {
    let t_copy_link = translations.copy_link;
    let t_copy_path = translations.copy_path;

    let copy_icon = icons::copy_link(translations.copy);
    let failure_icon = icons::failure(translations.failed);
    let success_icon = icons::success(translations.copied);

    let render = |local_path: &str, live_url: &str| -> String {
        formatdoc!(r#"
            <div class="path_widget">
                <div>
                    <a href="{live_url}" target="_blank">{live_url}</a>
                    <button class="action" data-copy="{live_url}">
                        <span class="icon">{copy_icon}</span>
                        {t_copy_link}
                    </button>
                </div>
                <div>
                    <span>{local_path}</span>
                    <button class="action" data-copy="{local_path}">
                        <span class="icon">{copy_icon}</span>
                        {t_copy_path}
                    </button>
                </div>
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
    };

    if path.is_empty() {
        let live_url = match &site.base_url {
            Some(hyper_url) => hyper_url.value.to_string(),
            None => format!("https://example.com/")
        };
        let local_path = context.site_dir.join("");

        render(&local_path.to_string_lossy(), &live_url)
    } else {
        let path_with_trailing_slash = Path::new(path).join("");
        let live_url = match &site.base_url {
            Some(hyper_url) => hyper_url.value.join(&path_with_trailing_slash.to_string_lossy()).unwrap().to_string(),
            None => format!("https://example.com/{}", path_with_trailing_slash.display())
        };
        let local_path = context.site_dir.join(path).join("");

        render(&local_path.to_string_lossy(), &live_url)
    }
}

pub fn preview_button(path: &str, translations: &Translations) -> String {
    let t_preview = translations.preview;
    let preview_icon = icons::preview(t_preview);

    formatdoc!(r#"
        <form action="/build/{path}" method="post" target="_blank">
            <button class="action">{preview_icon} {t_preview}</button>
        </form>
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

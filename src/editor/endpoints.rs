use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Form, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

pub mod collection;
pub mod convert;
mod build;
mod deploy;
pub mod index;
pub mod jobs;
pub mod playlist;
mod processing;
pub mod site;
pub mod video;

pub use build::build;
pub use deploy::deploy;
pub use jobs::jobs;
pub use processing::processing;

use crate::{Context, Language, ThemeKind};
use crate::editor;

#[derive(Deserialize)]
pub struct LanguageForm {
    language: String,
    redirect: String
}

pub async fn change_editor_language(
    context: Data<Arc<Context>>,
    form: Form<LanguageForm>,
) -> Redirect {
    *context.editor_language.lock().unwrap() = Language::from_code(&form.language);
    editor::persist_config(&form.language);

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .body(include_bytes!("assets/favicon.png").as_slice())
}

pub async fn font() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("font/woff2")
        .body(include_bytes!("assets/roboto-v30-latin_latin-ext-regular.woff2").as_slice())
}

pub async fn scripts(context: Data<Arc<Context>>) -> HttpResponse {
    let static_js = include_str!("assets/scripts.js");

    let translations = context.get_editor_language().translations;

    // We selectively provide only those translations to the runtime js code
    // that are absolutely required, handpicking them here.
    let t_idle = translations.idle;
    let t_processing = translations.processing;
    let translations_js = formatdoc!(r#"
        const t = {{
            idle: '{t_idle}',
            processing: '{t_processing}'
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

pub async fn splash_image() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(include_bytes!("assets/splash.jpg").as_slice())
}

pub async fn styles(context: Data<Arc<Context>>) -> HttpResponse {
    let theme_vars = match &context.editor_theme.kind {
        ThemeKind::Adaptive { dark, light } => {
            let dark_vars = dark.print();
            let light_vars = light.print();

            formatdoc!(r#"
                body {{
                    {dark_vars}
                }}
                @media (prefers-color-scheme: light) {{
                    body {{
                        {light_vars}
                    }}
                }}
            "#)
        }
        ThemeKind::Static(vars) => {
            let only_vars = vars.print();

            formatdoc!(r#"
                body {{
                    {only_vars}
                }}
            "#)
        }
    };

    let static_css = include_str!("assets/styles.css");
    let css = formatdoc!(r#"
        {theme_vars}
        {static_css}
    "#);

    HttpResponse::Ok()
        .content_type("text/css")
        .body(css)
}
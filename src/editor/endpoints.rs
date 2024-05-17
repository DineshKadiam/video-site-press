use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Form, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

pub mod collection;
pub mod convert;
mod build;
pub mod deploy;
pub mod deployment;
pub mod index;
pub mod jobs;
pub mod playlist;
mod processing;
pub mod site;
pub mod video;

pub use build::build;
pub use deployment::deployment;
pub use jobs::jobs;
pub use processing::processing;

use crate::{Context, EditorConfig, Language, Theme, ThemeKind};

#[derive(Deserialize)]
pub struct EditorConfigForm {
    language_code: String,
    redirect: String,
    theme_key: String
}

pub async fn change_editor_config(
    context: Data<Arc<Context>>,
    form: Form<EditorConfigForm>,
) -> Redirect {
    *context.editor_language.lock().unwrap() = Language::from_code(&form.language_code);
    *context.editor_theme.lock().unwrap() = Theme::from_key(&form.theme_key).unwrap_or_else(|_| Theme::adaptive());

    let editor_config = EditorConfig::new(&form.language_code, &form.theme_key);

    editor_config.persist();

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

pub async fn splash_logo() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(include_bytes!("assets/splash_logo.svg").as_slice())
}

pub async fn styles(context: Data<Arc<Context>>) -> HttpResponse {
    let theme = context.get_editor_theme();

    let theme_vars = match &theme.kind {
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
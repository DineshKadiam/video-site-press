use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

pub mod collection;
pub mod convert;
mod build;
mod deploy;
pub mod index;
pub mod playlist;
mod processing;
pub mod site;
pub mod video;

pub use build::build;
pub use deploy::deploy;
pub use processing::processing;

use crate::{Context, ThemeKind};

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

pub async fn scripts() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!("assets/scripts.js"))
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
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, Font, ThemeKind};

const FONT_ELEMENTS_SELECTOR: &str = "body, button, input, select, textarea";

pub async fn styles(context: Data<Arc<Context>>) -> HttpResponse {
    let theme = context.get_editor_theme();

    let default_family = Font::Default.family();
    let fonts_css = formatdoc!(r#"
        {FONT_ELEMENTS_SELECTOR} {{
            font-family: {default_family};
        }}
    "#);

    let splash_vars = theme.print_splash_vars();

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

    let static_css = include_str!("../assets/styles.css");
    let css = formatdoc!(r#"
        {splash_vars}
        {theme_vars}
        {fonts_css}
        {static_css}
    "#);

    HttpResponse::Ok()
        .content_type("text/css")
        .body(css)
}
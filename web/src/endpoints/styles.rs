// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

use hyper8_core::{Font, ThemeKind};

use crate::{Context, SessionState};

const FONT_ELEMENTS_SELECTOR: &str = "body, button, input, select, textarea";

pub async fn styles(
    context: Data<Arc<Context>>,
    session: Session
) -> HttpResponse {
    let theme = SessionState::theme(&context.server_mode, &session);

    let default_family = Font::Default.family();
    let fonts_css = formatdoc!(r#"
        {FONT_ELEMENTS_SELECTOR} {{
            font-family: {default_family};
        }}
    "#);

    let theme_vars = match theme.kind() {
        ThemeKind::Adaptive { dark, light } => {
            let dark_vars_hsl = dark.print_hsl();
            let dark_vars_oklch = dark.print_oklch();
            let light_vars_hsl = light.print_hsl();
            let light_vars_oklch = light.print_oklch();

            formatdoc!(r#"
                body {{
                    {dark_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {dark_vars_oklch}
                    }}
                }}
                @media (prefers-color-scheme: light) {{
                    body {{
                        {light_vars_hsl}
                    }}
                    @supports (color: oklch(0% 0 0)) {{
                        body {{
                            {light_vars_oklch}
                        }}
                    }}
                }}
            "#)
        }
        ThemeKind::Static(vars) => {
            let only_vars_hsl = vars.print_hsl();
            let only_vars_oklch = vars.print_oklch();

            formatdoc!(r#"
                body {{
                    {only_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {only_vars_oklch}
                    }}
                }}
            "#)
        }
    };

    let static_css = include_str!("../assets/editor.css");
    let css = formatdoc!(r#"
        {theme_vars}
        {fonts_css}
        {static_css}
    "#);

    HttpResponse::Ok()
        .content_type("text/css")
        .body(css)
}

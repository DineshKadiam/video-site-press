// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::web::Data;
use indoc::formatdoc;

use hyper8_core::{Font, Theme, ThemeKind};

use crate::{Context, EditorTheme, SessionState};

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
        ThemeKind::Adaptive { dark_theme, light_theme } => {
            let dark_theme_vars = dark_theme.css("dark-");
            let light_theme_vars = light_theme.css("light-");

            // In our theming we use the "space toggle" trick as a light-dark() function
            // with wider support across browsers, e.g.:
            //
            // 1. --dark: initial;
            // 2. --light: ;
            // 3. color: var(--dark, #000) var(--light, #fff);
            //
            // ... which resolves to "color: #000;" (and vice versa)
            //
            // In the interest of readability we mask the space toggle internals using
            // the ON/OFF constants below.
            //
            // For in-depth context on this trick see e.g.:
            // - https://kosalab.dev/posts/no-duplication-css-theming
            // - https://lea.verou.me/blog/2020/10/the-var-space-hack-to-toggle-multiple-values-with-one-custom-property/
            const ON: &str = "initial";
            const OFF: &str = " ";

            formatdoc!(r#"
                body {{ color-scheme: light dark; }}
                @media (prefers-color-scheme: dark) {{
                    body:not([data-pinned-theme]) {{
                        --dark: {ON};
                        --light: {OFF};
                    }}
                }}
                @media (prefers-color-scheme: light) {{
                    body:not([data-pinned-theme]) {{
                        --dark: {OFF};
                        --light: {ON};
                    }}
                }}
                body[data-pinned-theme="dark"] {{
                    color-scheme: dark;
                    --dark: {ON};
                    --light: {OFF};
                }}
                body[data-pinned-theme="light"] {{
                    color-scheme: light;
                    --dark: {OFF};
                    --light: {ON};
                }}
                @supports (color: oklch(0% 0% 0)) {{
                    body {{
                        {dark_theme_vars}
                        {light_theme_vars}
                        --acc: var(--dark, var(--dark-acc)) var(--light, var(--light-acc));
                        --b1: var(--dark, var(--dark-b1)) var(--light, var(--light-b1));
                        --b1-85: var(--dark, var(--dark-b1-85)) var(--light, var(--light-b1-85));
                        --b1-92: var(--dark, var(--dark-b1-92)) var(--light, var(--light-b1-92));
                        --b2: var(--dark, var(--dark-b2)) var(--light, var(--light-b2));
                        --b3: var(--dark, var(--dark-b3)) var(--light, var(--light-b3));
                        --bm: var(--dark, var(--dark-bm)) var(--light, var(--light-bm));
                        --checkerboard: var(--dark, var(--dark-checkerboard)) var(--light, var(--light-checkerboard));
                        --d1: var(--dark, var(--dark-d1)) var(--light, var(--light-d1));
                        --d1-15: var(--dark, var(--dark-d1-15)) var(--light, var(--light-d1-15));
                        --d1-70-mono: var(--dark, var(--dark-d1-70-mono)) var(--light, var(--light-d1-70-mono));
                        --d1-85-mono: var(--dark, var(--dark-d1-85-mono)) var(--light, var(--light-d1-85-mono));
                        --d1-mono: var(--dark, var(--dark-d1-mono)) var(--light, var(--light-d1-mono));
                        --d2: var(--dark, var(--dark-d2)) var(--light, var(--light-d2));
                        --d3: var(--dark, var(--dark-d3)) var(--light, var(--light-d3));
                        --d3-mono: var(--dark, var(--dark-d3-mono)) var(--light, var(--light-d3-mono));
                        --dm: var(--dark, var(--dark-dm)) var(--light, var(--light-dm));
                        --f1: var(--dark, var(--dark-f1)) var(--light, var(--light-f1));
                        --f1-5: var(--dark, var(--dark-f1-5)) var(--light, var(--light-f1-5));
                        --f2: var(--dark, var(--dark-f2)) var(--light, var(--light-f2));
                        --f3: var(--dark, var(--dark-f3)) var(--light, var(--light-f3));
                        --fm: var(--dark, var(--dark-fm)) var(--light, var(--light-fm));
                        --l1-mono: var(--dark, var(--dark-l1-mono)) var(--light, var(--light-l1-mono));
                        --l1-10-mono: var(--dark, var(--dark-l1-10-mono)) var(--light, var(--light-l1-10-mono));
                        --l2-mono: var(--dark, var(--dark-l2-mono)) var(--light, var(--light-l2-mono));
                        --l3-mono: var(--dark, var(--dark-l3-mono)) var(--light, var(--light-l3-mono));
                        --m: var(--dark, var(--dark-m)) var(--light, var(--light-m));
                    }}
                }}
            "#)
        }
        ThemeKind::Static(theme) => {
            let theme_vars = theme.css("");
            format!("@supports(color: oklch(0% 0 0)){{body{{{theme_vars}}}}}")
        }
    };

    let editor_extra_vars = {
        let extra_vars = theme.extra_vars();
        format!("@supports(color:oklch(0% 0% 0)){{body {{{extra_vars}}}}}")
    };

    let vars_hsl = Theme::HSL;
    let extra_vars_hsl = EditorTheme::HSL;

    let static_css = include_str!("../assets/editor.css");
    let css = formatdoc!(r#"
        body{{{vars_hsl}{extra_vars_hsl}}}
        {theme_vars}
        {editor_extra_vars}
        {fonts_css}
        {static_css}
    "#);

    HttpResponse::Ok()
        .content_type("text/css")
        .body(css)
}

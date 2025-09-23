// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Mutex};

use indoc::formatdoc;
use tauri::State;

use hyper8_core::{Theme, ThemeKind};

use crate::LauncherState;

/// Returns (css, icon)
#[tauri::command]
pub fn get_theme(
    state: State<'_, Arc<Mutex<LauncherState>>>
) -> (String, String) {
    let icon;
    let theme_vars;

    let theme = state.lock().unwrap().get_theme();

    icon = theme.icon();

    // TODO: Technically the same as in styles.rs - unify/reuse?
    theme_vars = match theme.kind() {
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

    let splash_vars = Theme::print_splash_vars();
    let css = format!("{splash_vars}{theme_vars}");

    (css, icon)
}

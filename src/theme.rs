// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::Translations;

mod color;
mod dark_deep;
mod dark_experimental;
mod dark_neutral;
mod light_experimental;
mod light_neutral;
mod theme_vars;

use color::Color;
use dark_deep::DARK_DEEP_VARS;
use dark_experimental::DARK_EXPERIMENTAL_VARS;
use dark_neutral::DARK_NEUTRAL_VARS;
use light_experimental::LIGHT_EXPERIMENTAL_VARS;
use light_neutral::LIGHT_NEUTRAL_VARS;
use theme_vars::ThemeVars;

#[derive(Clone, Debug)]
pub struct Theme {
    pub key: &'static str,
    pub kind: ThemeKind
}

#[derive(Clone, Debug)]
pub enum ThemeKind {
    Adaptive {
        dark: ThemeVars,
        light: ThemeVars
    },
    Static(ThemeVars)
}

impl Theme {
    const ADAPTIVE_EXPERIMENTAL: Theme = Theme {
        key: "adaptive_experimental",
        kind: ThemeKind::Adaptive {
            dark: DARK_EXPERIMENTAL_VARS,
            light: LIGHT_EXPERIMENTAL_VARS
        }
    };

    const ADAPTIVE_NEUTRAL: Theme = Theme {
        key: "adaptive_neutral",
        kind: ThemeKind::Adaptive {
            dark: DARK_NEUTRAL_VARS,
            light: LIGHT_NEUTRAL_VARS
        }
    };

    const DARK_DEEP: Theme = Theme {
        key: "dark_deep",
        kind: ThemeKind::Static(DARK_DEEP_VARS)
    };

    const DARK_EXPERIMENTAL: Theme = Theme {
        key: "dark_experimental",
        kind: ThemeKind::Static(DARK_EXPERIMENTAL_VARS)
    };

    const DARK_NEUTRAL: Theme = Theme {
        key: "dark_neutral",
        kind: ThemeKind::Static(DARK_NEUTRAL_VARS)
    };

    const LIGHT_EXPERIMENTAL: Theme = Theme {
        key: "light_experimental",
        kind: ThemeKind::Static(LIGHT_EXPERIMENTAL_VARS)
    };

    const LIGHT_NEUTRAL: Theme = Theme {
        key: "light_neutral",
        kind: ThemeKind::Static(LIGHT_NEUTRAL_VARS)
    };

    /// Splash gradient blend mode
    const SPLASH_BLEND: &'static str = "color";

    /// Splash background (furthest away/bottom layer)
    const SPLASH_BG: &'static str = "#9d1a57";

    /// Splash foreground (closest/top layer)
    const SPLASH_FG: &'static str = "#fff";

    /// Splash Northeast corner gradient color
    const SPLASH_NE: &'static str = "#43437d";

    /// Splash Northwest corner gradient color
    const SPLASH_NW: &'static str = "#cc0a2e";

    /// Splash Southeast corner gradient color
    const SPLASH_SE: &'static str = "#9b0000";

    /// Splash Southwest corner gradient color
    const SPLASH_SW: &'static str = "#f7dc00";

    pub fn from_key(key: &str) -> Result<Theme, String> {
        match key {
            "adaptive_experimental" => Ok(Theme::ADAPTIVE_EXPERIMENTAL),
            "adaptive_neutral" => Ok(Theme::ADAPTIVE_NEUTRAL),
            "dark_deep" => Ok(Theme::DARK_DEEP),
            "dark_experimental" => Ok(Theme::DARK_EXPERIMENTAL),
            "dark_neutral" => Ok(Theme::DARK_NEUTRAL),
            "light_experimental" => Ok(Theme::LIGHT_EXPERIMENTAL),
            "light_neutral" => Ok(Theme::LIGHT_NEUTRAL),
            _ => Err(format!("Unknown theme key '{key}' (known keys are 'adaptive_neutral', 'dark_neutral', 'light_neutral' and 'dark_experimental')"))
        }
    }

    pub fn print_splash_vars(&self) -> String {
        // TODO: Port slash to oklch as well eventually
        let splash_blend = Theme::SPLASH_BLEND;
        let splash_bg = Theme::SPLASH_BG;
        let splash_fg = Theme::SPLASH_FG;
        let splash_ne = Theme::SPLASH_NE;
        let splash_nw = Theme::SPLASH_NW;
        let splash_se = Theme::SPLASH_SE;
        let splash_sw = Theme::SPLASH_SW;

        formatdoc!(r#"
            body {{
                --splash-blend: {splash_blend};
                --splash-bg: {splash_bg};
                --splash-fg: {splash_fg};
                --splash-ne: {splash_ne};
                --splash-nw: {splash_nw};
                --splash-se: {splash_se};
                --splash-sw: {splash_sw};
            }}
        "#)
    }

    pub fn variants(translations: &Translations) -> [(&str, String); 7] {
        let t_adaptive = translations.adaptive;
        let t_dark = translations.dark;
        let t_light = translations.light;
        let t_neutral = translations.neutral;

        // We intentionally don't translate Experimental for now - probably/maybe
        // the "extended" eventual set of themes will be lined up with proper names
        // and only the neutral base theme will remain translated as a generic term.
        [
            ("adaptive_experimental", format!("{t_adaptive} Experimental")),
            ("adaptive_neutral", format!("{t_adaptive} {t_neutral}")),
            ("dark_deep", format!("{t_dark} Deep")),
            ("dark_experimental", format!("{t_dark} Experimental")),
            ("dark_neutral", format!("{t_dark} {t_neutral}")),
            ("light_experimental", format!("{t_light} Experimental")),
            ("light_neutral", format!("{t_light} {t_neutral}"))
        ]
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::ADAPTIVE_NEUTRAL
    }
}

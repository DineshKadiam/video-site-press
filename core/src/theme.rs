// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_translations::Translations;

mod color;
mod dark_cool;
mod dark_deep;
mod dark_neutral;
mod dark_warm;
mod light_cool;
mod light_deep;
mod light_neutral;
mod light_warm;
mod theme_vars;

use color::Color;
use dark_cool::DARK_COOL_VARS;
use dark_deep::DARK_DEEP_VARS;
use dark_neutral::DARK_NEUTRAL_VARS;
use dark_warm::DARK_WARM_VARS;
use light_cool::LIGHT_COOL_VARS;
use light_deep::LIGHT_DEEP_VARS;
use light_neutral::LIGHT_NEUTRAL_VARS;
use light_warm::LIGHT_WARM_VARS;
use theme_vars::ThemeVars;

pub use dark_neutral::DARK_NEUTRAL_VARS as GENERIC_DARK_THEME_VARS;
pub use light_neutral::LIGHT_NEUTRAL_VARS as GENERIC_LIGHT_THEME_VARS;

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
    const ADAPTIVE_COOL: Theme = Theme {
        key: "adaptive_cool",
        kind: ThemeKind::Adaptive {
            dark: DARK_COOL_VARS,
            light: LIGHT_COOL_VARS
        }
    };

    const ADAPTIVE_DEEP: Theme = Theme {
        key: "adaptive_deep",
        kind: ThemeKind::Adaptive {
            dark: DARK_DEEP_VARS,
            light: LIGHT_DEEP_VARS
        }
    };

    const ADAPTIVE_NEUTRAL: Theme = Theme {
        key: "adaptive_neutral",
        kind: ThemeKind::Adaptive {
            dark: DARK_NEUTRAL_VARS,
            light: LIGHT_NEUTRAL_VARS
        }
    };

    const ADAPTIVE_WARM: Theme = Theme {
        key: "adaptive_warm",
        kind: ThemeKind::Adaptive {
            dark: DARK_WARM_VARS,
            light: LIGHT_WARM_VARS
        }
    };

    const DARK_COOL: Theme = Theme {
        key: "dark_cool",
        kind: ThemeKind::Static(DARK_COOL_VARS)
    };

    const DARK_DEEP: Theme = Theme {
        key: "dark_deep",
        kind: ThemeKind::Static(DARK_DEEP_VARS)
    };

    const DARK_NEUTRAL: Theme = Theme {
        key: "dark_neutral",
        kind: ThemeKind::Static(DARK_NEUTRAL_VARS)
    };

    const DARK_WARM: Theme = Theme {
        key: "dark_warm",
        kind: ThemeKind::Static(DARK_WARM_VARS)
    };

    const LIGHT_COOL: Theme = Theme {
        key: "light_cool",
        kind: ThemeKind::Static(LIGHT_COOL_VARS)
    };

    const LIGHT_DEEP: Theme = Theme {
        key: "light_deep",
        kind: ThemeKind::Static(LIGHT_DEEP_VARS)
    };

    const LIGHT_NEUTRAL: Theme = Theme {
        key: "light_neutral",
        kind: ThemeKind::Static(LIGHT_NEUTRAL_VARS)
    };

    const LIGHT_WARM: Theme = Theme {
        key: "light_warm",
        kind: ThemeKind::Static(LIGHT_WARM_VARS)
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
            "adaptive_cool" => Ok(Theme::ADAPTIVE_COOL),
            "adaptive_deep" => Ok(Theme::ADAPTIVE_DEEP),
            "adaptive_neutral" => Ok(Theme::ADAPTIVE_NEUTRAL),
            "adaptive_warm" => Ok(Theme::ADAPTIVE_WARM),
            "dark_cool" => Ok(Theme::DARK_COOL),
            "dark_deep" => Ok(Theme::DARK_DEEP),
            "dark_neutral" => Ok(Theme::DARK_NEUTRAL),
            "dark_warm" => Ok(Theme::DARK_WARM),
            "light_cool" => Ok(Theme::LIGHT_COOL),
            "light_deep" => Ok(Theme::LIGHT_DEEP),
            "light_neutral" => Ok(Theme::LIGHT_NEUTRAL),
            "light_warm" => Ok(Theme::LIGHT_WARM),
            _ => Err(format!("Unknown theme key '{key}' (known keys are 'adaptive_cool', 'adaptive_deep', 'adaptive_neutral', 'adaptive_warm', 'dark_cool', 'dark_deep', 'dark_neutral', 'dark_warm', 'light_cool', 'light_neutral' and 'light_warm')"))
        }
    }

    pub fn print_splash_vars() -> String {
        // TODO: Port splash to oklch as well eventually
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

    pub fn variants(translations: &Translations) -> [(&str, String); 12] {
        let t_adaptive = translations.adaptive;
        let t_cool = translations.cool;
        let t_dark = translations.dark;
        let t_deep = translations.deep;
        let t_light = translations.light;
        let t_neutral = translations.neutral;
        let t_warm = translations.warm;

        [
            ("adaptive_cool", format!("{t_adaptive} {t_cool}")),
            ("adaptive_deep", format!("{t_adaptive} {t_deep}")),
            ("adaptive_neutral", format!("{t_adaptive} {t_neutral}")),
            ("adaptive_warm", format!("{t_adaptive} {t_warm}")),
            ("dark_cool", format!("{t_dark} {t_cool}")),
            ("dark_deep", format!("{t_dark} {t_deep}")),
            ("dark_neutral", format!("{t_dark} {t_neutral}")),
            ("dark_warm", format!("{t_dark} {t_warm}")),
            ("light_cool", format!("{t_light} {t_cool}")),
            ("light_deep", format!("{t_light} {t_deep}")),
            ("light_neutral", format!("{t_light} {t_neutral}")),
            ("light_warm", format!("{t_light} {t_warm}"))
        ]
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::ADAPTIVE_NEUTRAL
    }
}

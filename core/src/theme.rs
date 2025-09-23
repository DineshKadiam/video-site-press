// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;
use serde_derive::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Theme {
    AdaptiveCool,
    AdaptiveDeep,
    AdaptiveNeutral,
    AdaptiveWarm,
    DarkCool,
    DarkDeep,
    DarkNeutral,
    DarkWarm,
    LightCool,
    LightDeep,
    LightNeutral,
    LightWarm
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

    pub fn key(&self) -> &'static str {
        match self {
            Theme::AdaptiveCool => "adaptive_cool",
            Theme::AdaptiveDeep => "adaptive_deep",
            Theme::AdaptiveNeutral => "adaptive_neutral",
            Theme::AdaptiveWarm => "adaptive_warm",
            Theme::DarkCool => "dark_cool",
            Theme::DarkDeep => "dark_deep",
            Theme::DarkNeutral => "dark_neutral",
            Theme::DarkWarm => "dark_warm",
            Theme::LightCool => "light_cool",
            Theme::LightDeep => "light_deep",
            Theme::LightNeutral => "light_neutral",
            Theme::LightWarm => "light_warm"
        }
    }

    pub fn kind(&self) -> ThemeKind {
        match self {
            Theme::AdaptiveCool => ThemeKind::Adaptive {
                dark: DARK_COOL_VARS,
                light: LIGHT_COOL_VARS
            },
            Theme::AdaptiveDeep => ThemeKind::Adaptive {
                dark: DARK_DEEP_VARS,
                light: LIGHT_DEEP_VARS
            },
            Theme::AdaptiveNeutral => ThemeKind::Adaptive {
                dark: DARK_NEUTRAL_VARS,
                light: LIGHT_NEUTRAL_VARS
            },
            Theme::AdaptiveWarm => ThemeKind::Adaptive {
                dark: DARK_WARM_VARS,
                light: LIGHT_WARM_VARS
            },
            Theme::DarkCool => ThemeKind::Static(DARK_COOL_VARS),
            Theme::DarkDeep => ThemeKind::Static(DARK_DEEP_VARS),
            Theme::DarkNeutral => ThemeKind::Static(DARK_NEUTRAL_VARS),
            Theme::DarkWarm => ThemeKind::Static(DARK_WARM_VARS),
            Theme::LightCool => ThemeKind::Static(LIGHT_COOL_VARS),
            Theme::LightDeep => ThemeKind::Static(LIGHT_DEEP_VARS),
            Theme::LightNeutral => ThemeKind::Static(LIGHT_NEUTRAL_VARS),
            Theme::LightWarm => ThemeKind::Static(LIGHT_WARM_VARS)
        }
    }

    pub fn from_key(key: &str) -> Result<Theme, String> {
        match key {
            "adaptive_cool" => Ok(Theme::AdaptiveCool),
            "adaptive_deep" => Ok(Theme::AdaptiveDeep),
            "adaptive_neutral" => Ok(Theme::AdaptiveNeutral),
            "adaptive_warm" => Ok(Theme::AdaptiveWarm),
            "dark_cool" => Ok(Theme::DarkCool),
            "dark_deep" => Ok(Theme::DarkDeep),
            "dark_neutral" => Ok(Theme::DarkNeutral),
            "dark_warm" => Ok(Theme::DarkWarm),
            "light_cool" => Ok(Theme::LightCool),
            "light_deep" => Ok(Theme::LightDeep),
            "light_neutral" => Ok(Theme::LightNeutral),
            "light_warm" => Ok(Theme::LightWarm),
            _ => Err(format!("Unknown theme key '{key}' (known keys are 'adaptive_cool', 'adaptive_deep', 'adaptive_neutral', 'adaptive_warm', 'dark_cool', 'dark_deep', 'dark_neutral', 'dark_warm', 'light_cool', 'light_neutral' and 'light_warm')"))
        }
    }

    /// Renders an html and inline-css based icon that roughly indicates the
    /// color tone(s) of the theme
    pub fn icon(&self) -> String {
        match self.kind() {
            ThemeKind::Adaptive { dark, light } => {
                let dark_inline_vars = dark.preview_gradient_vars("d-");
                let light_inline_vars = light.preview_gradient_vars("l-");

                format!(r#"
                    <div style="{dark_inline_vars}{light_inline_vars}background: linear-gradient(45deg, var(--l-a), var(--l-b) 49.999%, var(--d-a) 50%, var(--d-b)); border-radius: .2rem; height: 1em; width: 1em;"></div>
                "#)
            }
            ThemeKind::Static(vars) => {
                let inline_vars = vars.preview_gradient_vars("");

                format!(r#"
                    <div style="{inline_vars}background: linear-gradient(45deg, var(--a), var(--b)); border-radius: .2rem; height: 1em; width: 1em;"></div>
                "#)
            }
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

    pub fn variants(translations: &Translations) -> [(Theme, String); 12] {
        let t_adaptive = translations.adaptive;
        let t_cool = translations.cool;
        let t_dark = translations.dark;
        let t_deep = translations.deep;
        let t_light = translations.light;
        let t_neutral = translations.neutral;
        let t_warm = translations.warm;

        [
            (Theme::AdaptiveCool, format!("{t_adaptive} {t_cool}")),
            (Theme::AdaptiveDeep, format!("{t_adaptive} {t_deep}")),
            (Theme::AdaptiveNeutral, format!("{t_adaptive} {t_neutral}")),
            (Theme::AdaptiveWarm, format!("{t_adaptive} {t_warm}")),
            (Theme::DarkCool, format!("{t_dark} {t_cool}")),
            (Theme::DarkDeep, format!("{t_dark} {t_deep}")),
            (Theme::DarkNeutral, format!("{t_dark} {t_neutral}")),
            (Theme::DarkWarm, format!("{t_dark} {t_warm}")),
            (Theme::LightCool, format!("{t_light} {t_cool}")),
            (Theme::LightDeep, format!("{t_light} {t_deep}")),
            (Theme::LightNeutral, format!("{t_light} {t_neutral}")),
            (Theme::LightWarm, format!("{t_light} {t_warm}"))
        ]
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::AdaptiveNeutral
    }
}

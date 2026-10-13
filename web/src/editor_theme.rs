// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::indoc;
use serde_derive::{Deserialize, Serialize};

use hyper8_core::{
    Oklch,
    ThemeKind,
    ThemePaint,
    ThemeParams
};
use hyper8_translations::Translations;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum EditorTheme {
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

pub struct EditorExtraVars {
    pub danger: Oklch,
    pub danger_hover: Oklch,
    pub f_create: Oklch,
    pub f_delete: Oklch,
    pub f_update: Oklch,
    pub warning_b: Oklch,
    pub warning_f: Oklch
}

impl EditorExtraVars {
    pub fn adaptive_css() -> String {
        let mut declarations = Vec::new();

        let mut add = |key| {
            let declaration = format!("--{key}: var(--dark, var(--dark-{key})) var(--light, var(--light-{key}));");
            declarations.push(declaration)
        };

        add("danger");
        add("danger-hover");
        add("f-create");
        add("f-delete");
        add("f-update");
        add("warning-b");
        add("warning-f");

        declarations.join("")
    }

    pub fn css(&self, prefix: &str) -> String {
        let mut declarations = Vec::new();

        let mut add = |name, value| {
            let css = format!("--{prefix}{name}:{value};");
            declarations.push(css);
        };

        add("danger", self.danger.css());
        add("danger-hover", self.danger_hover.css());
        add("f-create", self.f_create.css());
        add("f-delete", self.f_delete.css());
        add("f-update", self.f_update.css());
        add("warning-b", self.warning_b.css());
        add("warning-f", self.warning_f.css());

        declarations.join("")
    }
}

impl EditorTheme {
    const DARK_COOL_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 12.0, 370.0),
        danger_hover: Oklch::lch_raw(56.0, 14.0, 370.0),
        f_create: Oklch::lch_raw(88.0, 25.0, 153.0),
        f_delete: Oklch::lch_raw(74.0, 20., 35.0),
        f_update: Oklch::lch_raw(95.0, 19.0, 123.0),
        warning_b: Oklch::lch_raw(32.0, 3.0, 364.0),
        warning_f: Oklch::lch_raw(100.0, 0.0, 0.0)
    };

    const DARK_DEEP_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(40.0, 13.0, 370.0),
        danger_hover: Oklch::lch_raw(50.0, 17.0, 372.0),
        f_create: Oklch::lch_raw(88.0, 25.0, 153.0),
        f_delete: Oklch::lch_raw(74.0, 20.0, 35.0),
        f_update: Oklch::lch_raw(95.0, 19.0, 123.0),
        warning_b: Oklch::lch_raw(25.0, 4.0, 384.0),
        warning_f: Oklch::lch_raw(100.0, 0.0, 0.0)
    };

    const DARK_NEUTRAL_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 20.0, 374.0),
        danger_hover: Oklch::lch_raw(56.0, 20.0, 374.0),
        f_create: Oklch::lch_raw(88.0, 25.0, 153.0),
        f_delete: Oklch::lch_raw(74.0, 2.0, 35.0),
        f_update: Oklch::lch_raw(95.0, 19.0, 123.0),
        warning_b: Oklch::lch_raw(29.0, 3.0, 380.0),
        warning_f: Oklch::lch_raw(100.0, 0.0, 0.0)
    };

    const DARK_WARM_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 15.0, 24.0),
        danger_hover: Oklch::lch_raw(56.0, 18.0, 24.0),
        f_create: Oklch::lch_raw(88.0, 25.0, 153.0),
        f_delete: Oklch::lch_raw(74.0, 20.0, 35.0),
        f_update: Oklch::lch_raw(95.0, 19.0, 123.0),
        warning_b: Oklch::lch_raw(35.0, 5.0, 400.0),
        warning_f: Oklch::lch_raw(100.0, 0.0, 0.0)
    };

    const LIGHT_COOL_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 12.0, 370.0),
        danger_hover: Oklch::lch_raw(56.0, 14.0, 370.0),
        f_create: Oklch::lch_raw(44.0, 11.0, 179.0),
        f_delete: Oklch::lch_raw(50.0, 23.0, 354.0),
        f_update: Oklch::lch_raw(50.0, 22.0, 302.0),
        warning_b: Oklch::lch_raw(0.94, 5.0, 97.0),
        warning_f: Oklch::lch_raw(0.0, 0.0, 0.0)
    };

    const LIGHT_DEEP_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 13.0, 365.0),
        danger_hover: Oklch::lch_raw(54.0, 17.0, 372.0),
        f_create: Oklch::lch_raw(44.0, 11.0, 179.0),
        f_delete: Oklch::lch_raw(50.0, 23.0, 354.0),
        f_update: Oklch::lch_raw(50.0, 22.0, 302.0),
        warning_b: Oklch::lch_raw(97.0, 6.0, 70.0),
        warning_f: Oklch::lch_raw(0.0, 0.0, 0.0)
    };

    const LIGHT_NEUTRAL_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 20.0, 374.0),
        danger_hover: Oklch::lch_raw(56.0, 20.0, 374.0),
        f_create: Oklch::lch_raw(44.0, 11.0, 179.0),
        f_delete: Oklch::lch_raw(50.0, 23.0, 354.0),
        f_update: Oklch::lch_raw(50.0, 22.0, 302.0),
        warning_b: Oklch::lch_raw(97.0, 6.0, 70.0),
        warning_f: Oklch::lch_raw(0.0, 0.0, 0.0)
    };

    const LIGHT_WARM_EXTRA: EditorExtraVars = EditorExtraVars {
        danger: Oklch::lch_raw(45.0, 15.0, 24.0),
        danger_hover: Oklch::lch_raw(56.0, 18.0, 24.0),
        f_create: Oklch::lch_raw(44.0, 11.0, 179.0),
        f_delete: Oklch::lch_raw(50.0, 23.0, 354.0),
        f_update: Oklch::lch_raw(50.0, 22.0, 302.0),
        warning_b: Oklch::lch_raw(90.0, 7.0, 80.0),
        warning_f: Oklch::lch_raw(0.0, 0.0, 0.0)
    };

    pub const HSL: &str = indoc!(r#"
        --checkerboard: url("data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIHZlcnNpb249IjEuMSIgdmlld0JveD0iMCAwIDY0IDY0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPgogICAgPHJlY3QgZmlsbD0iaHNsKDAgMCUgOTMuMyUpIiBoZWlnaHQ9IjMyIiB3aWR0aD0iMzIiIHg9IjAiIHk9IjAiLz4KICAgIDxyZWN0IGZpbGw9ImhzbCgwIDAlIDkzLjMlKSIgaGVpZ2h0PSIzMiIgd2lkdGg9IjMyIiB4PSIzMiIgeT0iMzIiLz4KICAgIDxyZWN0IGZpbGw9ImhzbCgwIDAlIDg2LjclKSIgaGVpZ2h0PSIzMiIgd2lkdGg9IjMyIiB4PSIwIiB5PSIzMiIvPgogICAgPHJlY3QgZmlsbD0iaHNsKDAgMCUgODYuNyUpIiBoZWlnaHQ9IjMyIiB3aWR0aD0iMzIiIHg9IjMyIiB5PSIwIi8+Cjwvc3ZnPgo=");
        --danger: hsl(336 100% 27.3%);
        --danger-hover: hsl(346 67.7% 48.3%);
        --f-create: hsl(173 100% 11%);
        --f-delete: hsl(321 100% 30%);
        --f-update: hsl(271 62% 48%);
        --warning-b: hsl(42 100% 90%);
        --warning-f: hsl(0 0% 0%);
    "#);

    pub fn extra_vars(&self) -> String {
        match self {
            EditorTheme::AdaptiveCool => {
                let dark_vars = Self::DARK_COOL_EXTRA.css("dark-");
                let light_vars = Self::LIGHT_COOL_EXTRA.css("light-");
                let adaptive_vars = EditorExtraVars::adaptive_css();

                format!("{dark_vars}{light_vars}{adaptive_vars}")
            }
            EditorTheme::AdaptiveDeep => {
                let dark_vars = Self::DARK_DEEP_EXTRA.css("dark-");
                let light_vars = Self::LIGHT_DEEP_EXTRA.css("light-");
                let adaptive_vars = EditorExtraVars::adaptive_css();

                format!("{dark_vars}{light_vars}{adaptive_vars}")
            }
            EditorTheme::AdaptiveNeutral => {
                let dark_vars = Self::DARK_NEUTRAL_EXTRA.css("dark-");
                let light_vars = Self::LIGHT_NEUTRAL_EXTRA.css("light-");
                let adaptive_vars = EditorExtraVars::adaptive_css();

                format!("{dark_vars}{light_vars}{adaptive_vars}")
            }
            EditorTheme::AdaptiveWarm => {
                let dark_vars = Self::DARK_WARM_EXTRA.css("dark-");
                let light_vars = Self::LIGHT_WARM_EXTRA.css("light-");
                let adaptive_vars = EditorExtraVars::adaptive_css();

                format!("{dark_vars}{light_vars}{adaptive_vars}")
            }
            EditorTheme::DarkCool => Self::DARK_COOL_EXTRA.css(""),
            EditorTheme::DarkDeep => Self::DARK_DEEP_EXTRA.css(""),
            EditorTheme::DarkNeutral => Self::DARK_NEUTRAL_EXTRA.css(""),
            EditorTheme::DarkWarm => Self::DARK_WARM_EXTRA.css(""),
            EditorTheme::LightCool => Self::LIGHT_COOL_EXTRA.css(""),
            EditorTheme::LightDeep => Self::LIGHT_DEEP_EXTRA.css(""),
            EditorTheme::LightNeutral => Self::LIGHT_NEUTRAL_EXTRA.css(""),
            EditorTheme::LightWarm => Self::LIGHT_WARM_EXTRA.css("")
        }
    }

    /// Renders an html and inline-css based icon that roughly indicates the
    /// color tone(s) of the theme
    pub fn icon(&self) -> String {
        match self.kind() {
            ThemeKind::Adaptive { dark_theme, light_theme } => {
                let dark_inline_vars = dark_theme.preview_gradient_vars("d-");
                let light_inline_vars = light_theme.preview_gradient_vars("l-");

                format!(r#"
                    <div style="{dark_inline_vars}{light_inline_vars}background: linear-gradient(45deg, var(--l-a), var(--l-b) 49.999%, var(--d-a) 50%, var(--d-b)); border-radius: .2rem; height: 1em; width: 1em;"></div>
                "#)
            }
            ThemeKind::Static(theme) => {
                let inline_vars = theme.preview_gradient_vars("");

                format!(r#"
                    <div style="{inline_vars}background: linear-gradient(45deg, var(--a), var(--b)); border-radius: .2rem; height: 1em; width: 1em;"></div>
                "#)
            }
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            EditorTheme::AdaptiveCool => "adaptive_cool",
            EditorTheme::AdaptiveDeep => "adaptive_deep",
            EditorTheme::AdaptiveNeutral => "adaptive_neutral",
            EditorTheme::AdaptiveWarm => "adaptive_warm",
            EditorTheme::DarkCool => "dark_cool",
            EditorTheme::DarkDeep => "dark_deep",
            EditorTheme::DarkNeutral => "dark_neutral",
            EditorTheme::DarkWarm => "dark_warm",
            EditorTheme::LightCool => "light_cool",
            EditorTheme::LightDeep => "light_deep",
            EditorTheme::LightNeutral => "light_neutral",
            EditorTheme::LightWarm => "light_warm"
        }
    }

    pub fn kind(&self) -> ThemeKind {
        match self {
            EditorTheme::AdaptiveCool => {
                ThemeKind::Adaptive {
                    dark_theme: ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_cool_vars())),
                    light_theme: ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_cool_vars()))
                }
            }
            EditorTheme::AdaptiveDeep => {
                ThemeKind::Adaptive {
                    dark_theme: ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_deep_vars())),
                    light_theme: ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_deep_vars()))
                }
            }
            EditorTheme::AdaptiveNeutral => {
                ThemeKind::Adaptive {
                    dark_theme: ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_neutral_vars())),
                    light_theme: ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_neutral_vars()))
                }
            }
            EditorTheme::AdaptiveWarm => {
                ThemeKind::Adaptive {
                    dark_theme: ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_warm_vars())),
                    light_theme: ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_warm_vars()))
                }
            }
            EditorTheme::DarkCool => {
                ThemeKind::Static(ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_cool_vars())))
            }
            EditorTheme::DarkDeep => {
                ThemeKind::Static(ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_deep_vars())))
            }
            EditorTheme::DarkNeutral => {
                ThemeKind::Static(ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_neutral_vars())))
            }
            EditorTheme::DarkWarm => {
                ThemeKind::Static(ThemeParams::DARK.compute_theme(&Some(ThemePaint::dark_warm_vars())))
            }
            EditorTheme::LightCool => {
                ThemeKind::Static(ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_cool_vars())))
            }
            EditorTheme::LightDeep => {
                ThemeKind::Static(ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_deep_vars())))
            }
            EditorTheme::LightNeutral => {
                ThemeKind::Static(ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_neutral_vars())))
            }
            EditorTheme::LightWarm => {
                ThemeKind::Static(ThemeParams::LIGHT.compute_theme(&Some(ThemePaint::light_warm_vars())))
            }
        }
    }

    pub fn variants(translations: &Translations) -> [(EditorTheme, String); 12] {
        let t_adaptive = translations.adaptive;
        let t_cool = translations.cool;
        let t_dark = translations.dark;
        let t_deep = translations.deep;
        let t_light = translations.light;
        let t_neutral = translations.neutral;
        let t_warm = translations.warm;

        [
            (EditorTheme::AdaptiveCool, format!("{t_adaptive} {t_cool}")),
            (EditorTheme::AdaptiveDeep, format!("{t_adaptive} {t_deep}")),
            (EditorTheme::AdaptiveNeutral, format!("{t_adaptive} {t_neutral}")),
            (EditorTheme::AdaptiveWarm, format!("{t_adaptive} {t_warm}")),
            (EditorTheme::DarkCool, format!("{t_dark} {t_cool}")),
            (EditorTheme::DarkDeep, format!("{t_dark} {t_deep}")),
            (EditorTheme::DarkNeutral, format!("{t_dark} {t_neutral}")),
            (EditorTheme::DarkWarm, format!("{t_dark} {t_warm}")),
            (EditorTheme::LightCool, format!("{t_light} {t_cool}")),
            (EditorTheme::LightDeep, format!("{t_light} {t_deep}")),
            (EditorTheme::LightNeutral, format!("{t_light} {t_neutral}")),
            (EditorTheme::LightWarm, format!("{t_light} {t_warm}"))
        ]
    }
}

impl TryFrom<&str> for EditorTheme {
    type Error = ();

    fn try_from(key: &str) -> Result<EditorTheme, ()> {
        match key {
            "adaptive_cool" => Ok(EditorTheme::AdaptiveCool),
            "adaptive_deep" => Ok(EditorTheme::AdaptiveDeep),
            "adaptive_neutral" => Ok(EditorTheme::AdaptiveNeutral),
            "adaptive_warm" => Ok(EditorTheme::AdaptiveWarm),
            "dark_cool" => Ok(EditorTheme::DarkCool),
            "dark_deep" => Ok(EditorTheme::DarkDeep),
            "dark_neutral" => Ok(EditorTheme::DarkNeutral),
            "dark_warm" => Ok(EditorTheme::DarkWarm),
            "light_cool" => Ok(EditorTheme::LightCool),
            "light_deep" => Ok(EditorTheme::LightDeep),
            "light_neutral" => Ok(EditorTheme::LightNeutral),
            "light_warm" => Ok(EditorTheme::LightWarm),
            _ => Err(())
        }
    }
}

impl Default for EditorTheme {
    fn default() -> EditorTheme {
        EditorTheme::AdaptiveNeutral
    }
}

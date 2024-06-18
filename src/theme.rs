use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use indoc::formatdoc;

/// We define/store:
/// - alpha, lightness and saturation as 0.0-1.0
/// - chroma as 0.0-infinity (sensible range defined by OKLCH color model)
/// - hue as 0-360
#[derive(Clone, Debug)]
pub enum Color {
    Chromatic {
        hsl: (u16, f32, f32),
        oklch: (f32, f32, u16)
    },
    Gradient {
        begin_hsl: (u16, f32, f32),
        begin_oklch: (f32, f32, u16),
        end_hsl: (u16, f32, f32),
        end_oklch: (f32, f32, u16)
    },
    Gray {
        hsl_l: f32,
        oklch_l: f32
    },
    GrayAlpha {
        a: f32,
        hsl_l: f32,
        oklch_l: f32
    }
}

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

/// Theme color variables are specified using notions of distance:
/// In dark color schemes the background is dark, while the foreground
/// is bright, and the other way round in bright color schemes.
/// Some things are constantly bright or dark in both dark and light
/// color schemes, these use aliases with dark/light terminology.
/// Most variables are just gray tones, but a few exist in colored variants
/// for setting an accent or indicating dangerous actions or such.
#[derive(Clone, Debug)]
pub struct ThemeVars {
    /// Action (like saving form input)
    pub action: Color,
    /// Action hover state
    pub action_hover: Color,
    /// Background 1 (furthest away/bottom layer)
    pub bg_1: Color,
    /// Background 2 (further away)
    pub bg_1_overlay: Color,
    /// Background 2 (further away) with some transparency
    pub bg_2: Color,
    /// Background 3 (far away)
    pub bg_3: Color,
    /// Between background and middleground
    pub bg_mg: Color,
    /// Dangerous action (like confirming deletion or deployment)
    pub danger: Color,
    /// Dangerous action hover state
    pub danger_hover: Color,
    /// Foreground 1 (closest/top layer)
    pub fg_1: Color,
    /// Foreground 2 (closer)
    pub fg_2: Color,
    /// Foreground 3 (close)
    pub fg_3: Color,
    /// Between foreground and middleground
    pub fg_mg: Color,
    /// Light 1 (brightest)
    pub light_1: &'static str,
    /// Middleground  (in between)
    pub mg: Color
}

impl Color {
    pub fn hsl(&self) -> String {
        match self {
            Color::Chromatic { hsl, .. } => {
                let h = hsl.0;
                let s = hsl.1 * 100.0;
                let l = hsl.2 * 100.0;

                format!("hsl({h} {s}% {l}%)")
            }
            Color::Gradient { begin_hsl, end_hsl, .. } => {
                let begin_h = begin_hsl.0;
                let begin_s = begin_hsl.1 * 100.0;
                let begin_l = begin_hsl.2 * 100.0;
                let end_h = end_hsl.0;
                let end_s = end_hsl.1 * 100.0;
                let end_l = end_hsl.2 * 100.0;

                format!("linear-gradient(180deg, hsl({begin_h} {begin_s}% {begin_l}%), hsl({end_h} {end_s}% {end_l}%))")
            }
            Color::Gray { hsl_l, .. } => {
                let l = hsl_l * 100.0;
                format!("hsl(0 0% {l}%)")
            }
            Color::GrayAlpha { a, hsl_l, .. } => {
                let a = a * 100.0;
                let l = hsl_l * 100.0;
                format!("hsl(0 0% {l}% / {a}%)")
            }
        }
    }

    pub fn oklch(&self) -> String {
        match self {
            Color::Chromatic { oklch, .. } => {
                let l = oklch.0 * 100.0;
                let c = oklch.1;
                let h = oklch.2;

                format!("oklch({l}% {c} {h})")
            }
            Color::Gradient { begin_oklch, end_oklch, .. } => {
                let begin_l = begin_oklch.0 * 100.0;
                let begin_c = begin_oklch.1;
                let begin_h = begin_oklch.2;
                let end_l = end_oklch.0 * 100.0;
                let end_c = end_oklch.1;
                let end_h = end_oklch.2;

                format!("linear-gradient(180deg, oklch({begin_l}% {begin_c} {begin_h}), oklch({end_l}% {end_c} {end_h}))")
            }
            Color::Gray { oklch_l, .. } => {
                let l = oklch_l * 100.0;
                format!("oklch({l}% 0 0)")
            }
            Color::GrayAlpha { a, oklch_l, .. } => {
                let l = oklch_l * 100.0;
                format!("oklch({l}% 0 0 / {a})")
            }
        }
    }
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

    pub fn adaptive() -> Theme {
        Theme {
            key: "adaptive",
            kind: ThemeKind::Adaptive {
                dark: ThemeVars::dark(),
                light: ThemeVars::light()
            }
        }
    }

    pub fn dark() -> Theme {
        Theme {
            key: "dark",
            kind: ThemeKind::Static(ThemeVars::dark())
        }
    }

    pub fn from_key(key: &str) -> Result<Theme, String> {
        match key {
            "adaptive" => Ok(Theme::adaptive()),
            "dark" => Ok(Theme::dark()),
            "light" => Ok(Theme::light()),
            _ => Err(format!("Unknown theme key '{key}' (known keys are 'adaptive', 'dark' and 'light')"))
        }
    }

    pub fn light() -> Theme {
        Theme {
            key: "light",
            kind: ThemeKind::Static(ThemeVars::light())
        }
    }

    pub fn print_splash_vars(&self) -> String {
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
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::adaptive()
    }
}

impl ThemeVars {
    fn checkerboard_base64(&self) -> String {
        // TODO: Switch these to oklch when we drop hsl at some point in the future
        let bg_2 = &self.bg_2.hsl();
        let bg_3 = &self.bg_3.hsl();

        let svg = formatdoc!(r#"
            <svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
                <rect fill="{bg_2}" height="32" width="32" x="0" y="0"/>
                <rect fill="{bg_2}" height="32" width="32" x="32" y="32"/>
                <rect fill="{bg_3}" height="32" width="32" x="0" y="32"/>
                <rect fill="{bg_3}" height="32" width="32" x="32" y="0"/>
            </svg>
        "#);

        BASE64_STANDARD.encode(svg)
    }

    pub fn dark() -> ThemeVars {
        ThemeVars {
            action: Color::Gradient {
                begin_hsl: (194, 1.0, 0.137),
                begin_oklch: (0.45, 0.2, 220),
                end_hsl: (226, 0.512, 0.42),
                end_oklch: (0.45, 0.14, 268)
            },
            action_hover: Color::Gradient {
                begin_hsl: (193, 1.0, 0.179),
                begin_oklch: (0.56, 0.24, 220),
                end_hsl: (229, 0.99, 0.615),
                end_oklch: (0.56, 0.24, 268)
            },
            bg_1: Color::Gray {
                hsl_l: 0.086,
                oklch_l: 0.2
            },
            bg_1_overlay: Color::GrayAlpha {
                a: 0.85,
                hsl_l: 0.086,
                oklch_l: 0.2
            },
            bg_2: Color::Gray {
                hsl_l: 0.149,
                oklch_l: 0.269
            },
            bg_3: Color::Gray {
                hsl_l: 0.2,
                oklch_l: 0.32
            },
            bg_mg: Color::Gray {
                hsl_l: 0.267,
                oklch_l: 0.387
            },
            danger: Color::Gradient {
                begin_hsl: (336, 1.0, 0.273),
                begin_oklch: (0.45, 0.2, 374),
                end_hsl: (355, 0.557, 0.371),
                end_oklch: (0.45, 0.14, 380)
            },
            danger_hover: Color::Gradient {
                begin_hsl: (346, 0.677, 0.483),
                begin_oklch: (0.56, 0.2, 374),
                end_hsl: (352, 0.67, 0.488),
                end_oklch: (0.56, 0.2, 380)
            },
            fg_1: Color::Gray {
                hsl_l: 1.0,
                oklch_l: 1.0
            },
            fg_2: Color::Gray {
                hsl_l: 0.788,
                oklch_l: 0.84
            },
            fg_3: Color::Gray {
                hsl_l: 0.698,
                oklch_l: 0.77
            },
            fg_mg: Color::Gray {
                hsl_l: 0.596,
                oklch_l: 0.68
            },
            light_1: "var(--fg-1)",
            mg: Color::Gray {
                hsl_l: 0.533,
                oklch_l: 0.627
            }
        }
    }

    pub fn light() -> ThemeVars {
        ThemeVars {
            action: Color::Gradient {
                begin_hsl: (194, 1.0, 0.137),
                begin_oklch: (0.45, 0.2, 220),
                end_hsl: (226, 0.512, 0.42),
                end_oklch: (0.45, 0.14, 268)
            },
            action_hover: Color::Gradient {
                begin_hsl: (193, 1.0, 0.179),
                begin_oklch: (0.56, 0.24, 220),
                end_hsl: (229, 0.99, 0.615),
                end_oklch: (0.56, 0.24, 268)
            },
            bg_1: Color::Gray {
                hsl_l: 1.0,
                oklch_l: 1.0
            },
            bg_1_overlay: Color::GrayAlpha {
                a: 0.85,
                hsl_l: 1.0,
                oklch_l: 1.0
            },
            bg_2: Color::Gray {
                hsl_l: 0.933,
                oklch_l: 0.95
            },
            bg_3: Color::Gray {
                hsl_l: 0.867,
                oklch_l: 0.9
            },
            bg_mg: Color::Gray {
                hsl_l: 0.8,
                oklch_l: 0.85
            },
            danger: Color::Gradient {
                begin_hsl: (336, 1.0, 0.273),
                begin_oklch: (0.45, 0.2, 374),
                end_hsl: (355, 0.557, 0.371),
                end_oklch: (0.45, 0.14, 380)
            },
            danger_hover: Color::Gradient {
                begin_hsl: (346, 0.677, 0.483),
                begin_oklch: (0.56, 0.2, 374),
                end_hsl: (352, 0.67, 0.488),
                end_oklch: (0.56, 0.2, 380)
            },
            fg_1: Color::Gray {
                hsl_l: 0.0,
                oklch_l: 0.0
            },
            fg_2: Color::Gray {
                hsl_l: 0.153,
                oklch_l: 0.272
            },
            fg_3: Color::Gray {
                hsl_l: 0.267,
                oklch_l: 0.386
            },
            fg_mg: Color::Gray {
                hsl_l: 0.4,
                oklch_l: 0.51
            },
            light_1: "var(--bg-1)",
            mg: Color::Gray {
                hsl_l: 0.533,
                oklch_l: 0.627
            }
        }
    }

    pub fn print_hsl(&self) -> String {
        let action = self.action.hsl();
        let action_hover = self.action_hover.hsl();
        let bg_1 = self.bg_1.hsl();
        let bg_1_overlay = self.bg_1_overlay.hsl();
        let bg_2 = self.bg_2.hsl();
        let bg_3 = self.bg_3.hsl();
        let bg_mg = self.bg_mg.hsl();
        let checkerboard_base64 = self.checkerboard_base64();
        let danger = self.danger.hsl();
        let danger_hover = self.danger_hover.hsl();
        let fg_1 = self.fg_1.hsl();
        let fg_2 = self.fg_2.hsl();
        let fg_3 = self.fg_3.hsl();
        let fg_mg = self.fg_mg.hsl();
        let light_1 = self.light_1;
        let mg = self.mg.hsl();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --bg-1: {bg_1};
            --bg-1-overlay: {bg_1_overlay};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --danger: {danger};
            --danger-hover: {danger_hover};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --fg-mg: {fg_mg};
            --light-1: {light_1};
            --mg: {mg};
        "#)
    }

    pub fn print_oklch(&self) -> String {
        let action = self.action.oklch();
        let action_hover = self.action_hover.oklch();
        let bg_1 = self.bg_1.oklch();
        let bg_1_overlay = self.bg_1_overlay.oklch();
        let bg_2 = self.bg_2.oklch();
        let bg_3 = self.bg_3.oklch();
        let bg_mg = self.bg_mg.oklch();
        let checkerboard_base64 = self.checkerboard_base64();
        let danger = self.danger.oklch();
        let danger_hover = self.danger_hover.oklch();
        let fg_1 = self.fg_1.oklch();
        let fg_2 = self.fg_2.oklch();
        let fg_3 = self.fg_3.oklch();
        let fg_mg = self.fg_mg.oklch();
        let light_1 = self.light_1;
        let mg = self.mg.oklch();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --bg-1: {bg_1};
            --bg-1-overlay: {bg_1_overlay};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --danger: {danger};
            --danger-hover: {danger_hover};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --fg-mg: {fg_mg};
            --light-1: {light_1};
            --mg: {mg};
        "#)
    }
}
// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use indoc::formatdoc;

use super::Color;

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
    pub b1: Color,
    /// Background 2 (further away) with some transparency
    pub b2: Color,
    /// Background 3 (far away)
    pub b3: Color,
    /// Between background and middleground
    pub bm: Color,
    /// Dark 1 (darkest dark)
    pub d1: Color,
    /// Dark 2 (medium dark)
    pub d2: Color,
    /// Dark 3 (brightest dark)
    pub d3: Color,
    /// Dangerous action (like confirming deletion or deployment)
    pub danger: Color,
    /// Dangerous action hover state
    pub danger_hover: Color,
    /// Foreground 1 (closest/top layer)
    pub f1: Color,
    /// Foreground 2 (closer)
    pub f2: Color,
    /// Foreground 3 (close)
    pub f3: Color,
    /// Between foreground and middleground
    pub fm: Color,
    /// Light 1 (brightest bright)
    pub l1: Color,
    /// Light 2 (medium bright)
    pub l2: Color,
    /// Light 3 (darkest bright)
    pub l3: Color,
    /// Middleground  (in between)
    pub m: Color,
    /// Warning background (like pointing out a feature that needs additional
    /// configuration to work)
    pub warning_b: Color,
    /// Warning foreground (like pointing out a feature that needs additional
    /// configuration to work)
    pub warning_f: Color
}

impl ThemeVars {
    fn checkerboard_base64(&self) -> String {
        // TODO: Switch these to oklch when we drop hsl at some point in the future
        let b2 = &self.b2.hsl();
        let b3 = &self.b3.hsl();

        let svg = formatdoc!(r#"
            <svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
                <rect fill="{b2}" height="32" width="32" x="0" y="0"/>
                <rect fill="{b2}" height="32" width="32" x="32" y="32"/>
                <rect fill="{b3}" height="32" width="32" x="0" y="32"/>
                <rect fill="{b3}" height="32" width="32" x="32" y="0"/>
            </svg>
        "#);

        BASE64_STANDARD.encode(svg)
    }

    pub fn preview_gradient_vars(&self, prefix: &str) -> String {
        let a_hsl = self.b3.hsl();
        let a_oklch = self.b3.oklch();
        let b_hsl = self.bm.hsl();
        let b_oklch = self.bm.oklch();

        let a = format!("--{prefix}a: {a_hsl}; --{prefix}a: {a_oklch};");
        let b = format!("--{prefix}b: {b_hsl}; --{prefix}b: {b_oklch};");

        format!("{a}{b}")
    }

    pub fn print_hsl(&self) -> String {
        let action = self.action.hsl();
        let action_hover = self.action_hover.hsl();
        let b1 = self.b1.hsl();
        let b1_85 = self.b1.hsla(85.0);
        let b2 = self.b2.hsl();
        let b3 = self.b3.hsl();
        let bm = self.bm.hsl();
        let checkerboard_base64 = self.checkerboard_base64();
        let d1 = self.d1.hsl();
        let d1_15 = self.d1.hsla(15.0);
        let d1_85 = self.d1.hsla(85.0);
        let d2 = self.d2.hsl();
        let d3 = self.d3.hsl();
        let danger = self.danger.hsl();
        let danger_hover = self.danger_hover.hsl();
        let f1 = self.f1.hsl();
        let f2 = self.f2.hsl();
        let f3 = self.f3.hsl();
        let fm = self.fm.hsl();
        let l1 = self.l1.hsl();
        let l1_10 = self.l1.hsla(10.0);
        let l2 = self.l2.hsl();
        let l3 = self.l3.hsl();
        let m = self.m.hsl();
        let warning_b = self.warning_b.hsl();
        let warning_f = self.warning_f.hsl();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --b1: {b1};
            --b1-85: {b1_85};
            --b2: {b2};
            --b3: {b3};
            --bm: {bm};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --d1: {d1};
            --d1-15: {d1_15};
            --d1-85: {d1_85};
            --d2: {d2};
            --d3: {d3};
            --danger: {danger};
            --danger-hover: {danger_hover};
            --f1: {f1};
            --f2: {f2};
            --f3: {f3};
            --fm: {fm};
            --l1: {l1};
            --l1-10: {l1_10};
            --l2: {l2};
            --l3: {l3};
            --m: {m};
            --warning-b: {warning_b};
            --warning-f: {warning_f};
        "#)
    }

    pub fn print_oklch(&self) -> String {
        let action = self.action.oklch();
        let action_hover = self.action_hover.oklch();
        let b1 = self.b1.oklch();
        let b1_85 = self.b1.oklcha(85.0);
        let b2 = self.b2.oklch();
        let b3 = self.b3.oklch();
        let bm = self.bm.oklch();
        let checkerboard_base64 = self.checkerboard_base64();
        let d1 = self.d1.oklch();
        let d1_15 = self.d1.oklcha(15.0);
        let d1_85 = self.d1.oklcha(85.0);
        let d2 = self.d2.oklch();
        let d3 = self.d3.oklch();
        let danger = self.danger.oklch();
        let danger_hover = self.danger_hover.oklch();
        let f1 = self.f1.oklch();
        let f2 = self.f2.oklch();
        let f3 = self.f3.oklch();
        let fm = self.fm.oklch();
        let l1 = self.l1.oklch();
        let l1_10 = self.l1.oklcha(10.0);
        let l2 = self.l2.oklch();
        let l3 = self.l3.oklch();
        let m = self.m.oklch();
        let warning_b = self.warning_b.oklch();
        let warning_f = self.warning_f.oklch();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --b1: {b1};
            --b1-85: {b1_85};
            --b2: {b2};
            --b3: {b3};
            --bm: {bm};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --d1: {d1};
            --d1-15: {d1_15};
            --d1-85: {d1_85};
            --d2: {d2};
            --d3: {d3};
            --danger: {danger};
            --danger-hover: {danger_hover};
            --f1: {f1};
            --f2: {f2};
            --f3: {f3};
            --fm: {fm};
            --l1: {l1};
            --l1-10: {l1_10};
            --l2: {l2};
            --l3: {l3};
            --m: {m};
            --warning-b: {warning_b};
            --warning-f: {warning_f};
        "#)
    }
}

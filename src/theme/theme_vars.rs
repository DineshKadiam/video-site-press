// SPDX-FileCopyrightText: 2024 Simon Repp
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

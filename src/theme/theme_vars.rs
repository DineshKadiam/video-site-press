// SPDX-FileCopyrightText: 2024-2025 Simon Repp
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
    /// Background 2 (further away) with some transparency
    pub bg_2: Color,
    /// Background 3 (far away)
    pub bg_3: Color,
    /// Between background and middleground
    pub bg_mg: Color,
    /// Dark 1 (darkest dark)
    pub d_1: Color,
    /// Dark 2 (medium dark)
    pub d_2: Color,
    /// Dark 3 (brightest dark)
    pub d_3: Color,
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
    /// Light 1 (brightest bright)
    pub l_1: Color,
    /// Light 2 (medium bright)
    pub l_2: Color,
    /// Light 3 (darkest bright)
    pub l_3: Color,
    /// Middleground  (in between)
    pub mg: Color,
    /// Warning background (like pointing out a feature that needs additional
    /// configuration to work)
    pub warning_bg: Color,
    /// Warning foreground (like pointing out a feature that needs additional
    /// configuration to work)
    pub warning_fg: Color
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
        let bg_1_85 = self.bg_1.hsla(85.0);
        let bg_2 = self.bg_2.hsl();
        let bg_3 = self.bg_3.hsl();
        let bg_mg = self.bg_mg.hsl();
        let checkerboard_base64 = self.checkerboard_base64();
        let d_1 = self.d_1.hsl();
        let d_1_85 = self.d_1.hsla(85.0);
        let d_2 = self.d_2.hsl();
        let d_3 = self.d_3.hsl();
        let danger = self.danger.hsl();
        let danger_hover = self.danger_hover.hsl();
        let fg_1 = self.fg_1.hsl();
        let fg_2 = self.fg_2.hsl();
        let fg_3 = self.fg_3.hsl();
        let fg_mg = self.fg_mg.hsl();
        let l_1 = self.l_1.hsl();
        let l_2 = self.l_2.hsl();
        let l_3 = self.l_3.hsl();
        let mg = self.mg.hsl();
        let warning_bg = self.warning_bg.hsl();
        let warning_fg = self.warning_fg.hsl();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --bg-1: {bg_1};
            --bg-1-85: {bg_1_85};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --d-1: {d_1};
            --d-1-85: {d_1_85};
            --d-2: {d_2};
            --d-3: {d_3};
            --danger: {danger};
            --danger-hover: {danger_hover};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --fg-mg: {fg_mg};
            --l-1: {l_1};
            --l-2: {l_2};
            --l-3: {l_3};
            --mg: {mg};
            --warning-bg: {warning_bg};
            --warning-fg: {warning_fg};
        "#)
    }

    pub fn print_oklch(&self) -> String {
        let action = self.action.oklch();
        let action_hover = self.action_hover.oklch();
        let bg_1 = self.bg_1.oklch();
        let bg_1_85 = self.bg_1.oklcha(85.0);
        let bg_2 = self.bg_2.oklch();
        let bg_3 = self.bg_3.oklch();
        let bg_mg = self.bg_mg.oklch();
        let checkerboard_base64 = self.checkerboard_base64();
        let d_1 = self.d_1.oklch();
        let d_1_85 = self.d_1.oklcha(85.0);
        let d_2 = self.d_2.oklch();
        let d_3 = self.d_3.oklch();
        let danger = self.danger.oklch();
        let danger_hover = self.danger_hover.oklch();
        let fg_1 = self.fg_1.oklch();
        let fg_2 = self.fg_2.oklch();
        let fg_3 = self.fg_3.oklch();
        let fg_mg = self.fg_mg.oklch();
        let l_1 = self.l_1.oklch();
        let l_2 = self.l_2.oklch();
        let l_3 = self.l_3.oklch();
        let mg = self.mg.oklch();
        let warning_bg = self.warning_bg.oklch();
        let warning_fg = self.warning_fg.oklch();

        formatdoc!(r#"
            --action: {action};
            --action-hover: {action_hover};
            --bg-1: {bg_1};
            --bg-1-85: {bg_1_85};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --d-1: {d_1};
            --d-1-85: {d_1_85};
            --d-2: {d_2};
            --d-3: {d_3};
            --danger: {danger};
            --danger-hover: {danger_hover};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --fg-mg: {fg_mg};
            --l-1: {l_1};
            --l-2: {l_2};
            --l-3: {l_3};
            --mg: {mg};
            --warning-bg: {warning_bg};
            --warning-fg: {warning_fg};
        "#)
    }
}

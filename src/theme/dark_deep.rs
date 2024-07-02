// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

pub const DARK_DEEP_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (194, 1.0, 0.18),
        oklch: (0.36, 0.18, 230)
    },
    action_hover: Color::Chromatic {
        hsl: (193, 1.0, 0.24),
        oklch: (0.46, 0.22, 225)
    },
    bg_1: Color::Gray {
        hsl_l: 0.058,
        oklch_l: 0.168
    },
    bg_1_overlay: Color::GrayAlpha {
        a: 0.85,
        hsl_l: 0.058,
        oklch_l: 0.168
    },
    bg_2: Color::Gray {
        hsl_l: 0.121,
        oklch_l: 0.239
    },
    bg_3: Color::Gray {
        hsl_l: 0.16,
        oklch_l: 0.28
    },
    bg_mg: Color::Gray {
        hsl_l: 0.267,
        oklch_l: 0.387
    },
    danger: Color::Chromatic {
        hsl: (344, 0.61, 0.31),
        oklch: (0.40, 0.13, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (345, 0.63, 0.42),
        oklch: (0.50, 0.17, 372)
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
};

// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

pub const LIGHT_DEEP_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (196, 1.0, 0.18),
        oklch: (0.36, 0.18, 230)
    },
    action_hover: Color::Chromatic {
        hsl: (193, 1.0, 0.24),
        oklch: (0.46, 0.22, 225)
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
    danger: Color::Chromatic {
        hsl: (341, 0.51, 0.37),
        oklch: (0.45, 0.13, 365)
    },
    danger_hover: Color::Chromatic {
        hsl: (347, 0.55, 0.47),
        oklch: (0.54, 0.17, 372)
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
};

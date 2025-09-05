// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Gray {
    hsl_l: 0.058,
    oklch_l: 0.168
};

const D_2: Color = Color::Gray {
    hsl_l: 0.121,
    oklch_l: 0.239
};

const D_3: Color = Color::Gray {
    hsl_l: 0.16,
    oklch_l: 0.28
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Gray {
    hsl_l: 0.788,
    oklch_l: 0.84
};

const L_3: Color = Color::Gray {
    hsl_l: 0.698,
    oklch_l: 0.77
};

pub const DARK_DEEP_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (194, 1.0, 0.18),
        oklch: (0.36, 0.18, 230)
    },
    action_hover: Color::Chromatic {
        hsl: (193, 1.0, 0.24),
        oklch: (0.46, 0.22, 225)
    },
    bg_1: D_1,
    bg_2: D_2,
    bg_3: D_3,
    bg_mg: Color::Gray {
        hsl_l: 0.267,
        oklch_l: 0.387
    },
    d_1: D_1,
    d_2: D_2,
    d_3: D_3,
    danger: Color::Chromatic {
        hsl: (344, 0.61, 0.31),
        oklch: (0.40, 0.13, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (345, 0.63, 0.42),
        oklch: (0.50, 0.17, 372)
    },
    fg_1: L_1,
    fg_2: L_2,
    fg_3: L_3,
    fg_mg: Color::Gray {
        hsl_l: 0.596,
        oklch_l: 0.68
    },
    l_1: L_1,
    l_2: L_2,
    l_3: L_3,
    mg: Color::Gray {
        hsl_l: 0.533,
        oklch_l: 0.627
    },
    warning_bg: Color::Chromatic {
        hsl: (5, 0.35, 0.15),
        oklch: (0.25, 0.04, 384)
    },
    warning_fg: L_1
};

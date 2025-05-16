// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D_2: Color = Color::Gray {
    hsl_l: 0.153,
    oklch_l: 0.272
};

const D_3: Color = Color::Gray {
    hsl_l: 0.267,
    oklch_l: 0.386
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Gray {
    hsl_l: 0.933,
    oklch_l: 0.95
};

const L_3: Color = Color::Gray {
    hsl_l: 0.867,
    oklch_l: 0.9
};

pub const LIGHT_NEUTRAL_VARS: ThemeVars = ThemeVars {
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
    bg_1: L_1,
    bg_2: L_2,
    bg_3: L_3,
    bg_mg: Color::Gray {
        hsl_l: 0.8,
        oklch_l: 0.85
    },
    d_1: D_1,
    d_2: D_2,
    d_3: D_3,
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
    fg_1: D_1,
    fg_2: D_2,
    fg_3: D_3,
    fg_mg: Color::Gray {
        hsl_l: 0.4,
        oklch_l: 0.51
    },
    l_1: L_1,
    l_2: L_2,
    l_3: L_3,
    mg: Color::Gray {
        hsl_l: 0.533,
        oklch_l: 0.627
    },
    warning_bg: Color::Chromatic {
        hsl: (42, 1.0, 0.9),
        oklch: (0.97, 0.06, 70)
    },
    warning_fg: D_1
};

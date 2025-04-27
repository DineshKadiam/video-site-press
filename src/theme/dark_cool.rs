// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Chromatic {
    hsl: (217, 0.19, 0.09),
    oklch: (0.2, 0.012, 260)
};

const D_2: Color = Color::Chromatic {
    hsl: (217, 0.12, 0.15),
    oklch: (0.269, 0.012, 260)
};

const D_3: Color = Color::Chromatic {
    hsl: (217, 0.09, 0.2),
    oklch: (0.32, 0.012, 260)
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Chromatic {
    hsl: (217, 0.12, 0.8),
    oklch: (0.84, 0.012, 260)
};

pub const DARK_COOL_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (225, 0.39, 0.4),
        oklch: (0.45, 0.1, 268)
    },
    action_hover: Color::Chromatic {
        hsl: (225, 0.46, 0.54),
        oklch: (0.56, 0.13, 268)
    },
    bg_1: D_1,
    bg_2: D_2,
    bg_3: D_3,
    bg_mg: Color::Chromatic {
        hsl: (217, 0.07, 0.27),
        oklch: (0.387, 0.012, 260)
    },
    d_1: D_1,
    d_2: D_2,
    d_3: D_3,
    danger: Color::Chromatic {
        hsl: (347, 0.46, 0.37),
        oklch: (0.45, 0.12, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (348, 0.43, 0.51),
        oklch: (0.56, 0.14, 370)
    },
    fg_1: L_1,
    fg_2: L_2,
    fg_3: Color::Chromatic {
        hsl: (217, 0.08, 0.71),
        oklch: (0.77, 0.012, 260)
    },
    fg_mg: Color::Chromatic {
        hsl: (217, 0.06, 0.6),
        oklch: (0.68, 0.012, 260)
    },
    l_1: L_1,
    l_2: L_2,
    mg: Color::Chromatic {
        hsl: (217, 0.05, 0.54),
        oklch: (0.627, 0.012, 260)
    },
    warning_bg: Color::Chromatic {
        hsl: (347, 0.17, 0.21),
        oklch: (0.32, 0.03, 364)
    },
    warning_fg: L_1
};

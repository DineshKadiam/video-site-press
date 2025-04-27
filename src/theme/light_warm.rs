// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D_2: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.15),
    oklch: (0.272, 0.004, 10)
};

const D_3: Color = Color::Chromatic {
    hsl: (354, 0.02, 0.27),
    oklch: (0.386, 0.004, 10)
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Chromatic {
    hsl: (354, 0.12, 0.94),
    oklch: (0.95, 0.004, 10)
};

pub const LIGHT_WARM_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (177, 1.0, 0.19),
        oklch: (0.45, 0.2, 190)
    },
    action_hover: Color::Chromatic {
        hsl: (178, 1.0, 0.26),
        oklch: (0.56, 0.24, 190)
    },
    bg_1: L_1,
    bg_2: L_2,
    bg_3: Color::Chromatic {
        hsl: (354, 0.06, 0.87),
        oklch: (0.9, 0.004, 10)
    },
    bg_mg: Color::Chromatic {
        hsl: (354, 0.04, 0.81),
        oklch: (0.85, 0.004, 10)
    },
    d_1: D_1,
    d_2: D_2,
    d_3: D_3,
    danger: Color::Chromatic {
        hsl: (358, 0.61, 0.37),
        oklch: (0.45, 0.15, 24)
    },
    danger_hover: Color::Chromatic {
        hsl: (359, 0.57, 0.5),
        oklch: (0.56, 0.18, 24)
    },
    fg_1: D_1,
    fg_2: D_2,
    fg_3: D_3,
    fg_mg: Color::Chromatic {
        hsl: (354, 0.02, 0.4),
        oklch: (0.51, 0.004, 10)
    },
    l_1: L_1,
    l_2: L_2,
    mg: Color::Chromatic {
        hsl: (354, 0.01, 0.54),
        oklch: (0.627, 0.004, 10)
    },
    warning_bg: Color::Chromatic {
        hsl: (37, 0.83, 0.82),
        oklch: (0.9, 0.07, 80)
    },
    warning_fg: D_1
};

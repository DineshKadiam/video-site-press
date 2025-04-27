// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Chromatic {
    hsl: (353, 0.07, 0.09),
    oklch: (0.2, 0.005, 10)
};

const D_2: Color = Color::Chromatic {
    hsl: (354, 0.04, 0.15),
    oklch: (0.269, 0.005, 10)
};

const D_3: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.2),
    oklch: (0.32, 0.005, 10)
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Chromatic {
    hsl: (354, 0.04, 0.8),
    oklch: (0.84, 0.005, 10)
};

pub const DARK_WARM_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (177, 1.0, 0.19),
        oklch: (0.45, 0.2, 190)
    },
    action_hover: Color::Chromatic {
        hsl: (178, 1.0, 0.26),
        oklch: (0.56, 0.24, 190)
    },
    bg_1: D_1,
    bg_2: D_2,
    bg_3: D_3,
    bg_mg: Color::Chromatic {
        hsl: (354, 0.03, 0.27),
        oklch: (0.387, 0.005, 10)
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
    fg_1: L_1,
    fg_2: L_2,
    fg_3: Color::Chromatic {
        hsl: (354, 0.03, 0.71),
        oklch: (0.77, 0.005, 10)
    },
    fg_mg: Color::Chromatic {
        hsl: (354, 0.02, 0.6),
        oklch: (0.68, 0.005, 10)
    },
    l_1: L_1,
    l_2: L_2,
    mg: Color::Chromatic {
        hsl: (354, 0.02, 0.54),
        oklch: (0.627, 0.005, 10)
    },
    warning_bg: Color::Chromatic {
        hsl: (15, 0.36, 0.23),
        oklch: (0.35, 0.05, 400)
    },
    warning_fg: L_1
};

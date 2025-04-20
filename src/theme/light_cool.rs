// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D_1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D_2: Color = Color::Chromatic {
    hsl: (217, 0.9, 0.16),
    oklch: (0.272, 0.009, 260)
};

const D_3: Color = Color::Chromatic {
    hsl: (217, 0.06, 0.27),
    oklch: (0.386, 0.009, 260)
};

const L_1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L_2: Color = Color::Chromatic {
    hsl: (217, 0.32, 0.94),
    oklch: (0.95, 0.009, 260)
};

pub const LIGHT_COOL_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (225, 0.39, 0.4),
        oklch: (0.45, 0.1, 268)
    },
    action_hover: Color::Chromatic {
        hsl: (225, 0.46, 0.54),
        oklch: (0.56, 0.13, 268)
    },
    bg_1: L_1,
    bg_2: L_2,
    bg_3: Color::Chromatic {
        hsl: (217, 0.15, 0.88),
        oklch: (0.9, 0.009, 260)
    },
    bg_mg: Color::Chromatic {
        hsl: (217, 0.1, 0.81),
        oklch: (0.85, 0.009, 260)
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
    fg_1: D_1,
    fg_2: D_2,
    fg_3: D_3,
    fg_mg: Color::Chromatic {
        hsl: (217, 0.04, 0.4),
        oklch: (0.51, 0.009, 260)
    },
    l_1: L_1,
    l_2: L_2,
    mg: Color::Chromatic {
        hsl: (217, 0.04, 0.54),
        oklch: (0.627, 0.009, 260)
    }
};

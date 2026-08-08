// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D2: Color = Color::Chromatic {
    hsl: (217, 0.9, 0.16),
    oklch: (0.272, 0.009, 260)
};

const D3: Color = Color::Chromatic {
    hsl: (217, 0.06, 0.27),
    oklch: (0.386, 0.009, 260)
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Chromatic {
    hsl: (217, 0.32, 0.94),
    oklch: (0.95, 0.009, 260)
};

const L3: Color = Color::Chromatic {
    hsl: (217, 0.15, 0.88),
    oklch: (0.9, 0.009, 260)
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
    b1: L1,
    b2: L2,
    b3: L3,
    bm: Color::Chromatic {
        hsl: (217, 0.1, 0.81),
        oklch: (0.85, 0.009, 260)
    },
    d1: D1,
    d2: D2,
    d3: D3,
    danger: Color::Chromatic {
        hsl: (347, 0.46, 0.37),
        oklch: (0.45, 0.12, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (348, 0.43, 0.51),
        oklch: (0.56, 0.14, 370)
    },
    f1: D1,
    f2: D2,
    f3: D3,
    fm: Color::Chromatic {
        hsl: (217, 0.04, 0.4),
        oklch: (0.51, 0.009, 260)
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Chromatic {
        hsl: (217, 0.04, 0.54),
        oklch: (0.627, 0.009, 260)
    },
    warning_b: Color::Chromatic {
        hsl: (50, 0.68, 0.87),
        oklch: (0.94, 0.05, 97)
    },
    warning_f: D1
};

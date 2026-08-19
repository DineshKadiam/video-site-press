// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Chromatic {
    hsl: (217, 0.19, 0.09),
    oklch: (0.2, 0.012, 260)
};

const D2: Color = Color::Chromatic {
    hsl: (217, 0.12, 0.15),
    oklch: (0.269, 0.012, 260)
};

const D3: Color = Color::Chromatic {
    hsl: (217, 0.09, 0.2),
    oklch: (0.32, 0.012, 260)
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Chromatic {
    hsl: (217, 0.12, 0.8),
    oklch: (0.84, 0.012, 260)
};

const L3: Color = Color::Chromatic {
    hsl: (217, 0.08, 0.71),
    oklch: (0.77, 0.012, 260)
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
    b1: D1,
    b2: D2,
    b3: D3,
    bm: Color::Chromatic {
        hsl: (217, 0.07, 0.27),
        oklch: (0.387, 0.012, 260)
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
    f_create: Color::Chromatic {
        hsl: (157, 1.0, 0.36),
        oklch: (0.88, 0.25, 153)
    },
    f_delete: Color::Chromatic {
        hsl: (12, 1.0, 0.68),
        oklch: (0.74, 0.2, 35)
    },
    f_update: Color::Chromatic {
        hsl: (64, 0.96, 0.63),
        oklch: (0.95, 0.19, 123)
    },
    f1: L1,
    f2: L2,
    f3: L3,
    fm: Color::Chromatic {
        hsl: (217, 0.06, 0.6),
        oklch: (0.68, 0.012, 260)
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Chromatic {
        hsl: (217, 0.05, 0.54),
        oklch: (0.627, 0.012, 260)
    },
    warning_b: Color::Chromatic {
        hsl: (347, 0.17, 0.21),
        oklch: (0.32, 0.03, 364)
    },
    warning_f: L1
};

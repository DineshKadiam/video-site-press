// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Gray {
    hsl_l: 0.058,
    oklch_l: 0.168
};

const D2: Color = Color::Gray {
    hsl_l: 0.121,
    oklch_l: 0.239
};

const D3: Color = Color::Gray {
    hsl_l: 0.16,
    oklch_l: 0.28
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Gray {
    hsl_l: 0.788,
    oklch_l: 0.84
};

const L3: Color = Color::Gray {
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
    b1: D1,
    b2: D2,
    b3: D3,
    bm: Color::Gray {
        hsl_l: 0.267,
        oklch_l: 0.387
    },
    d1: D1,
    d2: D2,
    d3: D3,
    danger: Color::Chromatic {
        hsl: (344, 0.61, 0.31),
        oklch: (0.40, 0.13, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (345, 0.63, 0.42),
        oklch: (0.50, 0.17, 372)
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
    fm: Color::Gray {
        hsl_l: 0.596,
        oklch_l: 0.68
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Gray {
        hsl_l: 0.533,
        oklch_l: 0.627
    },
    warning_b: Color::Chromatic {
        hsl: (5, 0.35, 0.15),
        oklch: (0.25, 0.04, 384)
    },
    warning_f: L1
};

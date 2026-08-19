// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Gray {
    hsl_l: 0.086,
    oklch_l: 0.2
};

const D2: Color = Color::Gray {
    hsl_l: 0.149,
    oklch_l: 0.269
};

const D3: Color = Color::Gray {
    hsl_l: 0.2,
    oklch_l: 0.32
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

pub const DARK_NEUTRAL_VARS: ThemeVars = ThemeVars {
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
        hsl: (0, 0.21, 0.18),
        oklch: (0.29, 0.03, 380)
    },
    warning_f: L1
};

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D2: Color = Color::Gray {
    hsl_l: 0.153,
    oklch_l: 0.272
};

const D3: Color = Color::Gray {
    hsl_l: 0.267,
    oklch_l: 0.386
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Gray {
    hsl_l: 0.933,
    oklch_l: 0.95
};

const L3: Color = Color::Gray {
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
    b1: L1,
    b2: L2,
    b3: L3,
    bm: Color::Gray {
        hsl_l: 0.8,
        oklch_l: 0.85
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
        hsl: (173, 1.0, 0.11),
        oklch: (0.44, 0.11, 179)
    },
    f_delete: Color::Chromatic {
        hsl: (321, 1.0, 0.3),
        oklch: (0.5, 0.23, 354)
    },
    f_update: Color::Chromatic {
        hsl: (271, 0.62, 0.48),
        oklch: (0.5, 0.22, 302)
    },
    f1: D1,
    f2: D2,
    f3: D3,
    fm: Color::Gray {
        hsl_l: 0.4,
        oklch_l: 0.51
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Gray {
        hsl_l: 0.533,
        oklch_l: 0.627
    },
    warning_b: Color::Chromatic {
        hsl: (42, 1.0, 0.9),
        oklch: (0.97, 0.06, 70)
    },
    warning_f: D1
};

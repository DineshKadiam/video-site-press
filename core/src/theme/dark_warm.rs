// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Chromatic {
    hsl: (353, 0.07, 0.09),
    oklch: (0.2, 0.005, 10)
};

const D2: Color = Color::Chromatic {
    hsl: (354, 0.04, 0.15),
    oklch: (0.269, 0.005, 10)
};

const D3: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.2),
    oklch: (0.32, 0.005, 10)
};

const DM: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.27),
    oklch: (0.387, 0.005, 10)
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Chromatic {
    hsl: (354, 0.04, 0.8),
    oklch: (0.84, 0.005, 10)
};

const L3: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.71),
    oklch: (0.77, 0.005, 10)
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
    b1: D1,
    b2: D2,
    b3: D3,
    bm: DM,
    d1: D1,
    d2: D2,
    d3: D3,
    danger: Color::Chromatic {
        hsl: (358, 0.61, 0.37),
        oklch: (0.45, 0.15, 24)
    },
    danger_hover: Color::Chromatic {
        hsl: (359, 0.57, 0.5),
        oklch: (0.56, 0.18, 24)
    },
    dm: DM,
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
        hsl: (354, 0.02, 0.6),
        oklch: (0.68, 0.005, 10)
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Chromatic {
        hsl: (354, 0.02, 0.54),
        oklch: (0.627, 0.005, 10)
    },
    warning_b: Color::Chromatic {
        hsl: (15, 0.36, 0.23),
        oklch: (0.35, 0.05, 400)
    },
    warning_f: L1
};

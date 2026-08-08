// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

const D1: Color = Color::Gray {
    hsl_l: 0.0,
    oklch_l: 0.0
};

const D2: Color = Color::Chromatic {
    hsl: (354, 0.03, 0.15),
    oklch: (0.272, 0.004, 10)
};

const D3: Color = Color::Chromatic {
    hsl: (354, 0.02, 0.27),
    oklch: (0.386, 0.004, 10)
};

const L1: Color = Color::Gray {
    hsl_l: 1.0,
    oklch_l: 1.0
};

const L2: Color = Color::Chromatic {
    hsl: (354, 0.12, 0.94),
    oklch: (0.95, 0.004, 10)
};

const L3: Color = Color::Chromatic {
    hsl: (354, 0.06, 0.87),
    oklch: (0.9, 0.004, 10)
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
    b1: L1,
    b2: L2,
    b3: L3,
    bm: Color::Chromatic {
        hsl: (354, 0.04, 0.81),
        oklch: (0.85, 0.004, 10)
    },
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
    f1: D1,
    f2: D2,
    f3: D3,
    fm: Color::Chromatic {
        hsl: (354, 0.02, 0.4),
        oklch: (0.51, 0.004, 10)
    },
    l1: L1,
    l2: L2,
    l3: L3,
    m: Color::Chromatic {
        hsl: (354, 0.01, 0.54),
        oklch: (0.627, 0.004, 10)
    },
    warning_b: Color::Chromatic {
        hsl: (37, 0.83, 0.82),
        oklch: (0.9, 0.07, 80)
    },
    warning_f: D1
};

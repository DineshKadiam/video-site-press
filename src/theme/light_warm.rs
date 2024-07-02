// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

pub const LIGHT_WARM_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (177, 1.0, 0.19),
        oklch: (0.45, 0.2, 190)
    },
    action_hover: Color::Chromatic {
        hsl: (178, 1.0, 0.26),
        oklch: (0.56, 0.24, 190)
    },
    bg_1: Color::Gray {
        hsl_l: 1.0,
        oklch_l: 1.0
    },
    bg_1_overlay: Color::GrayAlpha {
        a: 0.85,
        hsl_l: 1.0,
        oklch_l: 1.0
    },
    bg_2: Color::Chromatic {
        hsl: (354, 0.12, 0.94),
        oklch: (0.95, 0.004, 10)
    },
    bg_3: Color::Chromatic {
        hsl: (354, 0.06, 0.87),
        oklch: (0.9, 0.004, 10)
    },
    bg_mg: Color::Chromatic {
        hsl: (354, 0.04, 0.81),
        oklch: (0.85, 0.004, 10)
    },
    danger: Color::Chromatic {
        hsl: (358, 0.61, 0.37),
        oklch: (0.45, 0.15, 24)
    },
    danger_hover: Color::Chromatic {
        hsl: (359, 0.57, 0.5),
        oklch: (0.56, 0.18, 24)
    },
    fg_1: Color::Gray {
        hsl_l: 0.0,
        oklch_l: 0.0
    },
    fg_2: Color::Chromatic {
        hsl: (354, 0.03, 0.15),
        oklch: (0.272, 0.004, 10)
    },
    fg_3: Color::Chromatic {
        hsl: (354, 0.02, 0.27),
        oklch: (0.386, 0.004, 10)
    },
    fg_mg: Color::Chromatic {
        hsl: (354, 0.02, 0.4),
        oklch: (0.51, 0.004, 10)
    },
    light_1: "var(--bg-1)",
    mg: Color::Chromatic {
        hsl: (354, 0.01, 0.54),
        oklch: (0.627, 0.004, 10)
    }
};

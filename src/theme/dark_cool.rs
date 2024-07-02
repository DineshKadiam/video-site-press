// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Color, ThemeVars};

pub const DARK_COOL_VARS: ThemeVars = ThemeVars {
    action: Color::Chromatic {
        hsl: (225, 0.39, 0.4),
        oklch: (0.45, 0.1, 268)
    },
    action_hover: Color::Chromatic {
        hsl: (225, 0.46, 0.54),
        oklch: (0.56, 0.13, 268)
    },
    bg_1: Color::Chromatic {
        hsl: (217, 0.19, 0.09),
        oklch: (0.2, 0.012, 260)
    },
    bg_1_overlay: Color::ChromaticAlpha {
        a: 0.85,
        hsl: (217, 0.19, 0.09),
        oklch: (0.2, 0.012, 260)
    },
    bg_2: Color::Chromatic {
        hsl: (217, 0.12, 0.15),
        oklch: (0.269, 0.012, 260)
    },
    bg_3: Color::Chromatic {
        hsl: (217, 0.09, 0.2),
        oklch: (0.32, 0.012, 260)
    },
    bg_mg: Color::Chromatic {
        hsl: (217, 0.07, 0.27),
        oklch: (0.387, 0.012, 260)
    },
    danger: Color::Chromatic {
        hsl: (347, 0.46, 0.37),
        oklch: (0.45, 0.12, 370)
    },
    danger_hover: Color::Chromatic {
        hsl: (348, 0.43, 0.51),
        oklch: (0.56, 0.14, 370)
    },
    fg_1: Color::Gray {
        hsl_l: 1.0,
        oklch_l: 1.0
    },
    fg_2: Color::Chromatic {
        hsl: (217, 0.12, 0.8),
        oklch: (0.84, 0.012, 260)
    },
    fg_3: Color::Chromatic {
        hsl: (217, 0.08, 0.71),
        oklch: (0.77, 0.012, 260)
    },
    fg_mg: Color::Chromatic {
        hsl: (217, 0.06, 0.6),
        oklch: (0.68, 0.012, 260)
    },
    light_1: "var(--fg-1)",
    mg: Color::Chromatic {
        hsl: (217, 0.05, 0.54),
        oklch: (0.627, 0.012, 260)
    }
};

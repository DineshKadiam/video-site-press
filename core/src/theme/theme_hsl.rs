// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::indoc;

use crate::Theme;

// This is the single, static HSL fallback theme we provide for browsers that
// don't support our (otherwise OKLCH-based) theming system.

// TODO: Possibly compress/proceduralize with a let mut declarations =
// Vec::new(); (...) declarations.push(...); approach

impl Theme {
    pub const HSL: &str = indoc!(r#"
        --acc: hsl(0 0% 100%);
        --b1: hsl(0 0% 100%);
        --b1-85: hsl(0 0% 100% / 85%);
        --b2: hsl(0 0% 93.3%);
        --b3: hsl(0 0% 86.7%);
        --bm: hsl(0 0% 80%);
        --d1: hsl(0 0% 0%);
        --d1-15: hsl(0 0% 0% / 15%);
        --d1-70-mono: hsl(0 0% 0% / 70%);
        --d1-85-mono: hsl(0 0% 0% / 85%);
        --d1-mono: hsl(0 0% 0%);
        --d2: hsl(0 0% 15.3%);
        --d3: hsl(0 0% 26.7%);
        --d3-mono: hsl(0 0% 26.7%);
        --dm: hsl(0 0% 40%);
        --f1: hsl(0 0% 0%);
        --f2: hsl(0 0% 15.3%);
        --f3: hsl(0 0% 26.7%);
        --fm: hsl(0 0% 40%);
        --l1-mono: hsl(0 0% 100%);
        --l1-10-mono: hsl(0 0% 100% / 10%);
        --l2-mono: hsl(0 0% 93.3%);
        --l3-mono: hsl(0 0% 86.7%);
        --m: hsl(0 0% 53.3%);
    "#);
}

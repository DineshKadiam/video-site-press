// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Degrees, Percent, ThemePaint};

impl ThemePaint {
    pub fn light_warm_vars() -> ThemePaint {
        ThemePaint {
            accent_brightening: None,
            accent_chroma: None,
            accent_hue: None,
            base_chroma: Some(Percent::new(false, 2)),
            base_hue: Some(Degrees::new(None, 10)),
            contrast: Some(Percent::new(false, 100)),
            layered_chroma: None
        }
    }
}

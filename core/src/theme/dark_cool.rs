// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Degrees, Percent, ThemePaint};

impl ThemePaint {
    pub fn dark_cool_vars() -> ThemePaint {
        ThemePaint {
            accent_brightening: None,
            accent_chroma: None,
            accent_hue: None,
            base_chroma: Some(Percent::new(false, 3)),
            base_hue: Some(Degrees::new(None, 260)),
            contrast: Some(Percent::new(false, 0)),
            layered_chroma: None
        }
    }
}

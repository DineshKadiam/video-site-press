// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{Context, Site, ThemeKind};

pub fn generate_embed_js(context: &Context) {
    fs::write(
        context.build_dir.join("embed.js"),
        include_str!("assets/embed.js")
    ).unwrap();
}

pub fn generate_site_js(context: &Context, site: &Site) {
    let adaptive_theme_js = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => include_str!("assets/adaptive_theme.js"),
        ThemeKind::Static(_) => ""
    };

    let site_js = include_str!("assets/site.js");

    let js = formatdoc!(r#"
        {adaptive_theme_js}
        {site_js}
    "#);

    fs::write(context.build_dir.join("site.js"), js).unwrap();
}

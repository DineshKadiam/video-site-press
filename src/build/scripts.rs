use std::fs;

use indoc::formatdoc;

use crate::{Context, Site, ThemeKind};

pub fn generate(context: &Context, site: &Site) {
    let static_js = include_str!("assets/scripts.js");

    let adaptive_theme = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => include_str!("assets/adaptive_theme.js"),
        ThemeKind::Static(_) => ""
    };

    let js = formatdoc!(r#"
        {adaptive_theme}
        {static_js}
    "#);

    fs::write(context.build_dir.join("scripts.js"), js).unwrap();
}
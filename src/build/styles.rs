use indoc::formatdoc;
use std::fs;

use crate::{Context, Site};

pub fn generate(context: &Context, site: &Site) {
    let static_css = include_str!("assets/styles.css");

    let accent_color = &site.theme.accent_color;
    let background_color = &site.theme.background_color;
    let text_color = &site.theme.text_color;

    let dynamic_css = formatdoc!(r#"
        :root {{
            --c-accent: {accent_color};
            --c-background: {background_color};
            --c-grey: #444;
            --c-green: #5bd217;
            --c-text: {text_color};
        }}
    "#);

    let css = formatdoc!(r#"
        {dynamic_css}
        {static_css}
    "#);

    fs::write(context.build_dir.join("styles.css"), css).unwrap();
}
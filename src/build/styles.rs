use std::fs;

use indoc::formatdoc;

use crate::{Context, Site, ThemeKind};

pub fn generate(context: &Context, site: &Site) {
    let static_css = include_str!("assets/styles.css");

    let theme_vars = match &site.theme.kind {
        ThemeKind::Adaptive { dark, light } => {
            let static_adaptive_theme_css = include_str!("assets/adaptive_theme.css");

            let dark_vars = dark.print();
            let light_vars = light.print();

            formatdoc!(r#"
                body {{
                    {dark_vars}
                }}
                body.light {{
                    {light_vars}
                }}
                {static_adaptive_theme_css}
            "#)
        }
        ThemeKind::Static(vars) => {
            let only_vars = vars.print();

            formatdoc!(r#"
                body {{
                    {only_vars}
                }}
            "#)
        }
    };

    let poster_aspect = site.poster_aspect.numeric();

    let css = formatdoc!(r#"
        :root {{
            --poster-aspect: {poster_aspect};
        }}
        {theme_vars}
        {static_css}
    "#);

    fs::write(context.build_dir.join("styles.css"), css).unwrap();
}
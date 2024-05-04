use std::fs;
use std::path::Path;

use indoc::formatdoc;

use crate::{Context, Font, Site, ThemeKind};

const FONT_ELEMENTS_SELECTOR: &str = "body, button";

pub fn generate(context: &Context, site: &Site) {
    let fonts_css = match &site.font {
        Font::Custom(file_name) => {
            let extension = Path::new(file_name).extension().unwrap().to_string_lossy().to_string();
            let target_file_name = format!("custom.{extension}");

            fs::copy(
                context.site_dir.join(file_name),
                context.build_dir.join(&target_file_name)
            ).unwrap();

            let family = site.font.family();

            formatdoc!(r#"
                @font-face {{
                    font-family: 'Custom';
                    font-style: normal;
                    font-weight: 400;
                    src: url('{target_file_name}') format('{extension}');
                }}
                {FONT_ELEMENTS_SELECTOR} {{
                    font-family: {family};
                }}
            "#)
        }
        Font::Default => {
            let family = site.font.family();

            formatdoc!(r#"
                {FONT_ELEMENTS_SELECTOR} {{
                    font-family: {family};
                }}
            "#)
        }
    };

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
        {fonts_css}
        {theme_vars}
        {static_css}
    "#);

    fs::write(context.build_dir.join("styles.css"), css).unwrap();
}
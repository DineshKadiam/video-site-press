// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use indoc::formatdoc;

use crate::{Context, Font, Site, ThemeKind};
use crate::util::url_safe_hash_base64;

use super::AssetHashes;

const FONT_ELEMENTS_SELECTOR: &str = "body, button";

pub fn generate_embed_css(asset_hashes: &mut AssetHashes, context: &Context, site: &Site) {
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

    let static_css = include_str!("assets/embed.css");

    let theme_vars = match &site.theme.kind {
        ThemeKind::Adaptive { dark, light } => {
            let dark_vars_hsl = dark.print_hsl();
            let dark_vars_oklch = dark.print_oklch();
            let light_vars_hsl = light.print_hsl();
            let light_vars_oklch = light.print_oklch();

            formatdoc!(r#"
                body {{
                    {dark_vars_hsl}
                }}
                body.light {{
                    {light_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {dark_vars_oklch}
                    }}
                    body.light {{
                        {light_vars_oklch}
                    }}
                }}
            "#)
        }
        ThemeKind::Static(vars) => {
            let only_vars_hsl = vars.print_hsl();
            let only_vars_oklch = vars.print_oklch();

            formatdoc!(r#"
                body {{
                    {only_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {only_vars_oklch}
                    }}
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

    asset_hashes.embed_css = Some(url_safe_hash_base64(&css));

    fs::write(context.build_dir.join("embed.css"), css).unwrap();
}

pub fn generate_site_css(asset_hashes: &mut AssetHashes, context: &Context, site: &Site) {
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

    let static_css = include_str!("assets/site.css");

    let theme_vars = match &site.theme.kind {
        ThemeKind::Adaptive { dark, light } => {
            let static_adaptive_theme_css = include_str!("assets/adaptive_theme.css");

            let dark_vars_hsl = dark.print_hsl();
            let dark_vars_oklch = dark.print_oklch();
            let light_vars_hsl = light.print_hsl();
            let light_vars_oklch = light.print_oklch();

            formatdoc!(r#"
                body {{
                    {dark_vars_hsl}
                }}
                body.light {{
                    {light_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {dark_vars_oklch}
                    }}
                    body.light {{
                        {light_vars_oklch}
                    }}
                }}
                {static_adaptive_theme_css}
            "#)
        }
        ThemeKind::Static(vars) => {
            let only_vars_hsl = vars.print_hsl();
            let only_vars_oklch = vars.print_oklch();

            formatdoc!(r#"
                body {{
                    {only_vars_hsl}
                }}
                @supports (color: oklch(0% 0 0)) {{
                    body {{
                        {only_vars_oklch}
                    }}
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

    asset_hashes.site_css = Some(url_safe_hash_base64(&css));

    fs::write(context.build_dir.join("site.css"), css).unwrap();
}

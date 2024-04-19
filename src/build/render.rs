use indoc::formatdoc;

use crate::{Site, ThemeKind};
use crate::build::Breadcrumb;
use crate::icons;

pub fn layout(
    root_prefix: &str,
    site: &Site,
    body: &str,
    title: &str,
    breadcrumbs: &[Breadcrumb]
) -> String {
    // TODO: Hash assets (cache expiration)
    let head = formatdoc!(r#"
        <head>
            <title>{title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <script defer src="{root_prefix}scripts.js"></script>
            <link rel="stylesheet" href="{root_prefix}styles.css">
        </head>
    "#);

    let breadcrumbs_rendered = breadcrumbs
        .iter()
        .enumerate()
        .map(|(index, breadcrumb)| {
            let href = "../".repeat(breadcrumbs.len() - 1 - index);
            let label = breadcrumb.label();
            format!(r#"<a href="{href}">{label}</a>"#)
        })
        .collect::<Vec<String>>()
        .join(" › ");

    let dark_icon = icons::dark(&site.language.translations.dark_color_scheme);
    let light_icon = icons::light(&site.language.translations.light_color_scheme);
    let theme_toggle = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => formatdoc!(r#"
            <div id="theme-toggles">
                <button class="theme_toggle" id="toggle-dark">{dark_icon}</button>
                <button class="theme_toggle" id="toggle-light">{light_icon}</button>
            </div>
        "#),
        ThemeKind::Static(_) => String::new()
    };

    let hyper8_version = env!("CARGO_PKG_VERSION");

    let logo_icon = icons::logo(&site.language.translations.hyper8_logo);
    let body = formatdoc!(r##"
        <body>
            <div class="layout">
                <div class="growing">
                    <div class="center">
                        <header>
                            {breadcrumbs_rendered}
                        </header>
                        {body}
                    </div>
                </div>
                <footer>
                    <a class="system_info" href="https://simonrepp.com/hyper8" target="_blank">
                        {logo_icon} Hyper 8 Video System {hyper8_version}
                    </a>
                    {theme_toggle}
                </footer>
            </div>
        </body>
    "##);

    let lang_code = &site.language.code;
    let dir_attribute = if site.language.is_rtl() { r#"dir="rtl""# } else { "" };

    formatdoc!(r#"
        <!doctype html>
        <html {dir_attribute} lang="{lang_code}">
            {head}
            {body}
        </html>
    "#)
}

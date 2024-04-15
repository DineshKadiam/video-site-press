use indoc::formatdoc;

use crate::{Site, ThemeKind};
use crate::build::Breadcrumb;
use crate::icons::LOGO_SVG;

const ICON_DARK: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m31.934 8.834c-3.9179 0.005803-7.8855 1.0068-11.518 3.1035-11.069 6.3911-14.869 20.575-8.4785 31.645 6.3911 11.069 20.575 14.871 31.645 8.4805h0.001953c11.069-6.3911 14.869-20.577 8.4785-31.646-3.9944-6.9178-11.033-10.997-18.453-11.523-0.55651-0.039474-1.1161-0.059423-1.6758-0.058594zm0.013672 2.1836c0.5065-0.001032 1.0121 0.017281 1.5156 0.052734 6.7142 0.47271 13.077 4.1669 16.701 10.443 5.7976 10.043 2.3646 22.852-7.6777 28.65-0.81094 0.46815-1.6406 0.86959-2.4824 1.2188-4.275-1.0509-10.322-6.9759-15.115-15.277-4.7914-8.2993-6.899-16.498-5.6738-20.727 0.72372-0.55489 1.4874-1.0745 2.2988-1.543 3.2953-1.9023 6.8881-2.8111 10.434-2.8184z"/>
</svg>
"#;

const ICON_LIGHT: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m29.68 3.6836v9.2344h4.6406v-9.2344h-4.6406zm-16.199 6.5156-3.2812 3.2832 6.5996 6.5957 3.2812-3.2812-6.5996-6.5977zm37.039 0-6.5996 6.5977 3.2812 3.2812 6.5996-6.5957-3.2812-3.2832zm-18.52 10.086a11.715 11.715 0 0 0-11.715 11.715 11.715 11.715 0 0 0 11.715 11.713 11.715 11.715 0 0 0 11.715-11.713 11.715 11.715 0 0 0-11.715-11.715zm-28.316 9.3926v4.6426h9.2324v-4.6426h-9.2324zm47.4 0v4.6426h9.2324v-4.6426h-9.2324zm-34.285 14.242-6.5996 6.5957 3.2812 3.2832 6.5996-6.5977-3.2812-3.2812zm30.402 0-3.2812 3.2812 6.5996 6.5977 3.2812-3.2832-6.5996-6.5957zm-17.521 7.1602v9.2344h4.6406v-9.2344h-4.6406z"/>
</svg>
"#;

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

    let theme_toggle = match &site.theme.kind {
        ThemeKind::Adaptive { .. } => formatdoc!(r#"
            <div id="theme-toggles">
                <button class="theme_toggle" id="toggle-dark">{ICON_DARK}</button>
                <button class="theme_toggle" id="toggle-light">{ICON_LIGHT}</button>
            </div>
        "#),
        ThemeKind::Static(_) => String::new()
    };

    let hyper8_version = env!("CARGO_PKG_VERSION");

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
                        {LOGO_SVG} Hyper 8 Video System {hyper8_version}
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

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{Site, ThemeKind};
use crate::build::Traversal;
use crate::icons;
use crate::util::html_escape_outside_attribute;

use super::GENERATOR_META;

pub struct Layout {
    clipboard_script: bool,
    /// <template> tags that are used in the scripting on the page, and
    /// appended at the bottom of the body to optimize loading.
    templates: String
}

impl Layout {
    pub fn add_clipboard_script(&mut self) {
        // TODO: Implement actual interpolation of script (and separate implementation as file)
        self.clipboard_script = true;
    }

    pub fn add_templates(&mut self, templates: &str) {
        self.templates.push_str(templates);
    }

    pub fn new() -> Layout {
        Layout {
            clipboard_script: false,
            templates: String::new()
        }
    }

    pub fn render(
        &self,
        body: &str,
        site: &Site,
        title: &str,
        traversal: &Traversal
    ) -> String {
        let generator_meta = GENERATOR_META;
        let root_prefix = traversal.prefix_asc();

        // TODO: Hash assets (cache expiration)

        let clipboard_script = match self.clipboard_script {
            true => format!(r#"<script defer src="{root_prefix}clipboard.js"></script>"#),
            false => String::new()
        };

        let head = formatdoc!(r#"
            <head>
                <title>{title}</title>
                <meta charset="utf-8">
                {generator_meta}
                <meta name="viewport" content="width=device-width, initial-scale=1">
                {clipboard_script}
                <script defer src="{root_prefix}site.js" type="module"></script>
                <link rel="stylesheet" href="{root_prefix}site.css">
            </head>
        "#);

        let breadcrumbs = traversal.parents(site)
            .iter()
            .map(|breadcrumb| {
                let breadcrumb_rendered = breadcrumb.step.breadcrumb(&breadcrumb.href, site);
                format!(r#"{breadcrumb_rendered} <span style="font-size: 1.6rem; position: relative; top: -.2rem;">›</span> "#)
            })
            .collect::<Vec<String>>()
            .join("");

        let dark_icon = icons::dark(site.language.translations.dark_color_scheme);
        let light_icon = icons::light(site.language.translations.light_color_scheme);
        let theme_toggle = match &site.theme.kind {
            ThemeKind::Adaptive { .. } => formatdoc!(r#"
                <div id="theme-toggles">
                    <button class="theme_toggle" id="toggle-dark">{dark_icon}</button>
                    <button class="theme_toggle" id="toggle-light">{light_icon}</button>
                </div>
            "#),
            ThemeKind::Static(_) => String::new()
        };

        let version_display = env!("HYPER8_VERSION_DISPLAY");

        let with_sidebar = if traversal.steps.len() > 1 { "with_sidebar" } else { "" };

        let e_site_title = html_escape_outside_attribute(site.title());

        let templates = &self.templates;

        let body = formatdoc!(r##"
            <body>
                <div class="layout {with_sidebar}">
                    <header>
                        <div class="breadcrumbs">
                            {breadcrumbs}
                        </div>
                        {theme_toggle}
                    </header>
                    <div class="growing">
                        <div class="center">
                            {body}
                        </div>
                    </div>
                    <footer>
                        <span>
                            {e_site_title}
                        </span>
                        <a class="system_info" href="https://simonrepp.com/hyper8" target="_blank">
                            Hyper 8 Video System {version_display}
                        </a>
                    </footer>
                </div>
                {templates}
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
}

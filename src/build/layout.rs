// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{OpenGraphMeta, Site, ThemeKind};
use crate::build::{AssetHashes, Traversal};
use crate::icons;
use crate::util::html_escape_outside_attribute;

use super::{META_GENERATOR, META_ROBOTS_NOINDEX_NOFOLLOW};

pub struct Layout {
    clipboard_script: bool,
    indexing: bool,
    /// <template> tags that are used in the scripting on the page, and
    /// appended at the bottom of the body to optimize loading.
    opengraph_meta: Option<OpenGraphMeta>,
    templates: String,
    video_script: bool
}

impl Layout {
    pub fn add_clipboard_script(&mut self) {
        self.clipboard_script = true;
    }

    pub fn add_opengraph_meta(&mut self, opengraph_meta: OpenGraphMeta) {
        self.opengraph_meta = Some(opengraph_meta);
    }

    pub fn add_templates(&mut self, templates: &str) {
        self.templates.push_str(templates);
    }

    pub fn add_video_script(&mut self) {
        self.video_script = true;
    }

    pub fn new() -> Layout {
        Layout {
            clipboard_script: false,
            indexing: true,
            opengraph_meta: None,
            templates: String::new(),
            video_script: false
        }
    }

    /// Call to enable meta tags that tell crawlers to not index the page.
    pub fn no_indexing(&mut self) {
        self.indexing = false;
    }

    pub fn render(
        &self,
        asset_hashes: &AssetHashes,
        body: &str,
        site: &Site,
        title: &str,
        traversal: &Traversal
    ) -> String {
        let meta_generator = META_GENERATOR;
        let meta_noindex_nofollow = if self.indexing { "" } else { META_ROBOTS_NOINDEX_NOFOLLOW };
        let root_prefix = traversal.prefix_asc();

        let adaptive_theme_script = if let ThemeKind::Adaptive { .. } = site.theme.kind {
            let adaptive_theme_js_asset_hash = AssetHashes::ADAPTIVE_THEME_JS;
            format!(r#"<script defer src="{root_prefix}adaptive_theme.js?{adaptive_theme_js_asset_hash}"></script>"#)
        } else {
            String::new()
        };

        let clipboard_script = if self.clipboard_script {
            let clipboard_js_asset_hash = AssetHashes::CLIPBOARD_JS;
            format!(r#"<script defer src="{root_prefix}clipboard.js?{clipboard_js_asset_hash}"></script>"#)
        } else {
            String::new()
        };

        let meta_opengraph = match &self.opengraph_meta {
            Some(meta) => meta.tags(site),
            None => String::new()
        };

        let site_css_asset_hash = asset_hashes.site_css.as_ref().unwrap();

        let video_script = if self.video_script {
            let video_js_asset_hash = AssetHashes::VIDEO_JS;
            format!(r#"<script defer src="{root_prefix}video.js?{video_js_asset_hash}" type="module"></script>"#)
        } else {
            String::new()
        };

        let head = formatdoc!(r#"
            <head>
                <title>{title}</title>
                <meta charset="utf-8">
                {meta_generator}
                {meta_noindex_nofollow}
                {meta_opengraph}
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <link rel="stylesheet" href="{root_prefix}site.css?{site_css_asset_hash}">
                {adaptive_theme_script}
                {clipboard_script}
                {video_script}
            </head>
        "#);

        let breadcrumbs = traversal.parents(site)
            .iter()
            .map(|breadcrumb| {
                let breadcrumb_rendered = breadcrumb.step.breadcrumb(&breadcrumb.href, site);
                format!(r#"{breadcrumb_rendered} <span style="font-size: 1.6rem; pointer-events: none; position: relative; top: -.2rem;">›</span> "#)
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

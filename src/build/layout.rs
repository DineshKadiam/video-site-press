// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{PlatformIntegrationMeta, Site, ThemeKind};
use crate::build::{AssetHashes, Traversal};
use crate::build::feeds::meta_link_tags;
use crate::build::widgets::{copy_button_icon_templates, player_icon_templates};
use crate::icons;
use crate::util::html_escape_outside_attribute;

use super::{META_GENERATOR, META_ROBOTS_NOINDEX_NOFOLLOW};

pub struct Layout {
    clipboard_script: bool,
    feed_links: bool,
    indexing: bool,
    platform_integration_meta: Option<PlatformIntegrationMeta>,
    video_script: bool
}

impl Layout {
    pub fn add_clipboard_script(&mut self) {
        self.clipboard_script = true;
    }

    pub fn add_feed_links(&mut self) {
        self.feed_links = true;
    }

    pub fn add_platform_integration_meta(&mut self, platform_integration_meta: PlatformIntegrationMeta) {
        self.platform_integration_meta = Some(platform_integration_meta);
    }

    pub fn add_video_script(&mut self) {
        self.video_script = true;
    }

    pub fn new() -> Layout {
        Layout {
            clipboard_script: false,
            feed_links: false,
            indexing: true,
            platform_integration_meta: None,
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
        let root_prefix = traversal.prefix_asc();
        let translations = &site.language.translations;

        let e_site_title = html_escape_outside_attribute(site.title());

        let mut extra_meta = String::new();

        if !self.indexing {
            extra_meta.push_str(META_ROBOTS_NOINDEX_NOFOLLOW);
        }

        if self.feed_links {
            extra_meta.push_str(&meta_link_tags(translations));
        }

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

        if let Some(meta) = &self.platform_integration_meta {
            extra_meta.push_str(&meta.tags(site));
        }

        let site_css_asset_hash = asset_hashes.site_css.as_ref().unwrap();

        let video_script = if self.video_script {
            let video_js_asset_hash = AssetHashes::VIDEO_JS;
            format!(r#"<script defer src="{root_prefix}video.js?{video_js_asset_hash}" type="module"></script>"#)
        } else {
            String::new()
        };

        let browse;
        let search;
        let navigation_script;
        if let Some(navigation_js_asset_hash) = &asset_hashes.navigation_js {
            let search_icon = icons::search(translations.search);
            let close_icon = icons::failure(translations.close);
            let t_search = translations.search;

            let t_browse = translations.browse;
            browse = formatdoc!(r#"
                <button class="browse">
                    {t_browse}
                </button>
            "#);

            search = formatdoc!(r#"
                <div class="search">
                    <div class="icon">{search_icon}</div>
                    <input autocomplete="off" placeholder="{t_search}" type="search">
                    <button>
                        {close_icon}
                    </button>
                </div>
                <div class="site_tree" data-root-prefix="{root_prefix}">
                    <div role="status"></div>
                    <div class="elements"></div>
                </div>
            "#);
            navigation_script = format!(r#"<script defer src="{root_prefix}navigation.js?{navigation_js_asset_hash}"></script>"#);
        } else {
            browse = String::new();
            search = String::new();
            navigation_script = String::new();
        };

        let head = formatdoc!(r#"
            <head>
                <title>{title}</title>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                {META_GENERATOR}
                {extra_meta}
                <link rel="stylesheet" href="{root_prefix}site.css?{site_css_asset_hash}">
                {adaptive_theme_script}
                {clipboard_script}
                {video_script}
                {navigation_script}
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

        let dark_icon = icons::dark(translations.dark_color_scheme);
        let light_icon = icons::light(translations.light_color_scheme);
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

        let mut templates = String::new();

        if self.clipboard_script {
            templates.push_str(&copy_button_icon_templates(translations));
        }

        if self.video_script {
            templates.push_str(&player_icon_templates(translations));
        }

        let logo_monochrome_icon = icons::logo_monochrome(None);
        let logo_polychrome_icon = icons::logo_polychrome(Some("Hyper 8"));
        let body = formatdoc!(r##"
            <body>
                <div class="layout">
                    <header>
                        <div class="navigation">
                            {logo_polychrome_icon}
                            <span class="title">{e_site_title}</span>
                            {browse}
                            {search}
                        </div>
                        <div>
                            {theme_toggle}
                        </div>
                    </header>
                    <div class="growing">
                        <div class="center">
                            {body}
                        </div>
                    </div>
                    <footer>
                        <div>
                            <span>
                                {e_site_title}
                            </span>
                            <a class="system_info" href="https://simonrepp.com/hyper8" target="_blank">
                                {logo_monochrome_icon} Hyper 8 Video System {version_display}
                            </a>
                        </div>
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

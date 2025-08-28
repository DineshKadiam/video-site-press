// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{PlatformIntegrationMeta, Site, ThemeKind};
use crate::build::{AssetHashes, Traversal};
use crate::build::feeds::meta_link_tags;
use crate::build::player::player_icon_templates;
use crate::build::widgets::copy_button_icon_templates;
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

        let video_script = if self.video_script {
            let video_js_asset_hash = AssetHashes::VIDEO_JS;
            format!(r#"<script defer src="{root_prefix}video.js?{video_js_asset_hash}" type="module"></script>"#)
        } else {
            String::new()
        };

        let mut templates = String::new();

        let navigation;
        let navigation_script;
        if let Some(navigation_js_asset_hash) = &asset_hashes.navigation_js {
            let browse_icon = icons::BROWSE;
            let close_icon = icons::failure(translations.close);
            let search_icon = icons::search(translations.search);
            let t_browse = translations.browse;
            let t_search = translations.search;

            navigation = formatdoc!(r#"
                <div class="navigation">
                    <button aria-label="{t_browse}" class="browse">
                        <span class="icon">{browse_icon}</span>
                        <span aria-hidden="true" class="label">{t_browse}</span>
                    </button>
                    <div class="search">
                        <span class="icon">{search_icon}</span>
                        <span aria-hidden="true" class="label">{t_search}</span>
                        <input autocomplete="off" type="search">
                        <button>
                            {close_icon}
                        </button>
                    </div>
                </div>
                <div class="site_tree">
                    <div role="status"></div>
                    <div class="elements"></div>
                </div>
            "#);

            navigation_script = format!(r#"<script defer src="{root_prefix}navigation.js?{navigation_js_asset_hash}"></script>"#);

            let chevron_right_icon = icons::CHEVRON_RIGHT;
            let navigation_templates = formatdoc!(r#"
                <template id="chevron_right_icon">
                    {chevron_right_icon}
                </template>
            "#);

            templates.push_str(&navigation_templates);
        } else {
            navigation = String::new();
            navigation_script = String::new();
        };

        let favicon_png_asset_hash = AssetHashes::FAVICON_PNG;
        let site_css_asset_hash = asset_hashes.site_css.as_ref().unwrap();

        let head = formatdoc!(r#"
            <head>
                <title>{title}</title>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                {META_GENERATOR}
                {extra_meta}
                <link href="{root_prefix}favicon.png?{favicon_png_asset_hash}" rel="icon" type="image/png">
                <link rel="stylesheet" href="{root_prefix}site.css?{site_css_asset_hash}">
                {adaptive_theme_script}
                {clipboard_script}
                {video_script}
                {navigation_script}
            </head>
        "#);

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

        if self.clipboard_script {
            templates.push_str(&copy_button_icon_templates(translations));
        }

        if self.video_script {
            templates.push_str(&player_icon_templates(translations));
        }

        let root_url = traversal.index_asc(site);
        let site_path = traversal.site_path().normalized();

        let key_icon = icons::KEY;
        let logo_monochrome_icon = icons::logo_monochrome(None);
        let logo_polychrome_icon = icons::logo_polychrome(Some("Hyper 8"));
        let t_adjust_playback_speed_cursor_above_playback_speed_indicator = translations.adjust_playback_speed_cursor_above_playback_speed_indicator;
        let t_arrow_left_right = translations.arrow_left_right;
        let t_clear_search_or_close_browse_panel = translations.clear_search_or_close_browse_panel;
        let t_close = translations.close;
        let t_focus_search_field = translations.focus_search_field;
        let t_enter_exit_fullscreen = translations.enter_exit_fullscreen;
        let t_jump_back_forward_five_seconds = translations.jump_back_forward_five_seconds;
        let t_mousewheel = translations.mousewheel;
        let t_mute_unmute_audio = translations.mute_unmute_audio;
        let t_navigation = translations.navigation;
        let t_open_browse_panel = translations.open_browse_panel;
        let t_play_pause_video = translations.play_pause_video;
        let t_shortcuts = translations.shortcuts;
        let t_space = translations.space;
        let t_video_player = translations.video_player;
        let body = formatdoc!(r##"
            <body>
                <script>
                    window.hyper8 = {{ rootPrefix: '{root_prefix}', sitePath: '{site_path}' }};
                </script>
                <div class="layout">
                    <header>
                        <a href="{root_url}">
                            {logo_polychrome_icon}
                            <span class="title">{e_site_title}</span>
                        </a>
                        {navigation}
                        {theme_toggle}
                    </header>
                    <div class="growing">
                        <div class="center">
                            {body}
                        </div>
                    </div>
                    <footer>
                        <a href="{root_url}">
                            {e_site_title}
                        </a>
                        <a href="https://simonrepp.com/hyper8" target="_blank">
                            {logo_monochrome_icon} Hyper 8 Video System {version_display}
                        </a>
                        <button class="shortcuts">
                            {key_icon} {t_shortcuts}
                        </button>
                    </footer>
                </div>
                <dialog class="shortcuts">
                    <h3>{t_navigation}</h3>
                    <ul>
                        <li>{t_open_browse_panel} <span>b</span></li>
                        <li>{t_focus_search_field} <span>s</span></li>
                        <li>{t_clear_search_or_close_browse_panel} <span>Esc</span></li>
                    </ul>
                    <h3>{t_video_player}</h3>
                    <ul>
                        <li>{t_enter_exit_fullscreen} <span>f</span></li>
                        <li>{t_mute_unmute_audio} <span>m</span></li>
                        <li>{t_jump_back_forward_five_seconds} <span>{t_arrow_left_right}</span></li>
                        <li>{t_play_pause_video} <span>{t_space}</span></li>
                        <li>{t_adjust_playback_speed_cursor_above_playback_speed_indicator} <span>{t_mousewheel}</span></li>
                    </ul>
                    <form method="dialog">
                        <button>{t_close}</button>
                    </form>
                </dialog>
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

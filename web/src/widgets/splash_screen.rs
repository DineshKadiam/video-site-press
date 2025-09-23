// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_core::Theme;
use hyper8_version::VERSION_ADAPTIVE_PATCH;

use crate::{Select, SessionState};

pub fn splash_screen(session_state: &SessionState) -> String {
    let translations = &session_state.language.translations;

    let mut select_language = Select::new("language_code", translations.language);
    for (code, label) in [("de", "Deutsch"), ("en", "English")] {
        select_language.option(code, label);
    }
    select_language.selected(&session_state.language.code);

    let mut select_theme = Select::new("theme_key", translations.theme);

    for (theme, label) in Theme::variants(translations) {
        select_theme.option(theme.key(), label);
    }
    select_theme.selected(session_state.theme.key());

    let link_icon = hyper8_icons::link(Some(translations.external_link));
    let t_documentation = translations.documentation;
    let t_close = translations.close;
    let t_report_a_bug = translations.report_a_bug;
    let t_version = translations.version;
    let t_website = translations.website;
    formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <div class="image_wrapper">
                    <img alt="Hyper 8 Video System" src="/splash_logo.svg?{VERSION_ADAPTIVE_PATCH}">
                    <a class="version" href="https://codeberg.org/simonrepp/hyper8/src/branch/main/CHANGELOG.md" target="_blank">
                        {link_icon}
                        <span>{t_version} {VERSION_ADAPTIVE_PATCH}</span>
                    </a>
                </div>
                <div class="pad">
                    <form action="/change-language-and-theme" method="post">
                        <input name="redirect" type="hidden">
                        <div class="form_split even">
                            {select_language}
                            {select_theme}
                        </div>
                    </form>
                    <ul autofocus>
                        <li>
                            <a href="https://simonrepp.com/hyper8" target="_blank">
                                {link_icon} {t_website}
                            </a>
                        </li>
                        <li>
                            <a href="https://simonrepp.com/hyper8/docs" target="_blank">
                                {link_icon} {t_documentation}
                            </a>
                        </li>
                        <li>
                            <a href="https://codeberg.org/simonrepp/hyper8/issues" target="_blank">
                                {link_icon} {t_report_a_bug}
                            </a>
                        </li>
                        <li>
                            <a href="https://liberapay.com/simonrepp" target="_blank">
                                {link_icon} Liberapay
                            </a>
                        </li>
                        <li>
                            <a href="https://ko-fi.com/simonrepp" target="_blank">
                                {link_icon} Ko-fi
                            </a>
                        </li>
                    </ul>
                    <form class="close" method="dialog">
                        <button autofocus>{t_close}</button>
                    </form>
                </div>
            </div>
        </dialog>
    "#)
}

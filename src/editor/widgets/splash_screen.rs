// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{Language, Theme};
use crate::editor::Select;
use crate::icons;

pub fn splash_screen(language: &Language, theme: Theme) -> String {
    let translations = &language.translations;

    let mut select_language = Select::new("language_code", translations.language);
    for (code, label) in [("de", "Deutsch"), ("en", "English")] {
        select_language.option(code, label);
    }
    select_language.selected(&language.code);

    let mut select_theme = Select::new("theme_key", translations.theme);

    for (key, label) in Theme::variants(translations) {
        select_theme.option(key, label);
    }
    select_theme.selected(theme.key);

    let external_icon = icons::external(translations.external_link);
    let version_adaptive = env!("HYPER8_VERSION_ADAPTIVE");
    let t_documentation = translations.documentation;
    let t_close = translations.close;
    let t_report_a_bug = translations.report_a_bug;
    let t_version = translations.version;
    let t_website = translations.website;
    formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <div class="image_wrapper">
                    <img alt="Hyper 8 Video System" src="/splash_logo.svg?{version_adaptive}">
                    <a class="version" href="https://codeberg.org/simonrepp/hyper8/src/branch/main/CHANGELOG.md" target="_blank">
                        {external_icon}
                        <span>{t_version} {version_adaptive}</span>
                    </a>
                </div>
                <div class="pad">
                    <form action="/change-editor-config" method="post">
                        <input name="redirect" type="hidden">
                        <div class="form_split even">
                            {select_language}
                            {select_theme}
                        </div>
                    </form>
                    <ul autofocus>
                        <li>
                            <a href="https://simonrepp.com/hyper8" target="_blank">
                                {external_icon} {t_website}
                            </a>
                        </li>
                        <li>
                            <a href="https://simonrepp.com/hyper8/docs" target="_blank">
                                {external_icon} {t_documentation}
                            </a>
                        </li>
                        <li>
                            <a href="https://codeberg.org/simonrepp/hyper8/issues" target="_blank">
                                {external_icon} {t_report_a_bug}
                            </a>
                        </li>
                        <li>
                            <a href="https://liberapay.com/simonrepp" target="_blank">
                                {external_icon} Liberapay
                            </a>
                        </li>
                        <li>
                            <a href="https://ko-fi.com/simonrepp" target="_blank">
                                {external_icon} Ko-fi
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

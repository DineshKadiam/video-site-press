// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{Language, Theme};
use crate::icons;

pub fn splash_screen(language: &Language, theme: Theme) -> String {
    let translations = &language.translations;

    let language_options = [("de", "Deutsch"), ("en", "English")]
        .into_iter()
        .map(|(code, label)| {
            let selected = if code == &language.code { "selected" } else { "" };
            format!(r#"<option {selected} value="{code}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let theme_options = [
        ("adaptive", translations.adaptive),
        ("dark", translations.dark),
        ("light", translations.light)
    ]
        .into_iter()
        .map(|(key, label)| {
            let selected = if &key == &theme.key { "selected" } else { "" };
            format!(r#"<option {selected} value="{key}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let external_icon = icons::external(translations.external_link);
    let hyper8_version = env!("CARGO_PKG_VERSION");
    let t_documentation = translations.documentation;
    let t_close = translations.close;
    let t_language = translations.language;
    let t_report_a_bug = translations.report_a_bug;
    let t_theme = translations.theme;
    let t_version = translations.version;
    let t_website = translations.website;
    formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <div class="image_wrapper">
                    <img alt="Hyper 8 Video System" src="/splash_logo.svg?{hyper8_version}">
                    <a class="version" href="https://codeberg.org/simonrepp/hyper8/src/branch/main/CHANGELOG.md" target="_blank">
                        {external_icon}
                        <span>{t_version} {hyper8_version}</span>
                    </a>
                </div>
                <div class="pad">
                    <form action="/change-editor-config" class="editor_config" method="post">
                        <input name="redirect" type="hidden">
                            <label for="language_code">{t_language}</label>
                            <select id="language_code" name="language_code">
                                {language_options}
                            </select>
                            <span style="display: inline-block; width: 1rem;"></span>
                            <label for="theme_key">{t_theme}</label>
                            <select id="theme_key" name="theme_key">
                                {theme_options}
                            </select>
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

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::AVAILABLE_LANGUAGES;
use hyper8_version::{VERSION_ADAPTIVE_PATCH, VERSION_WITH_PATCH};

use crate::LauncherState;

pub fn render(state: &LauncherState) -> String {
    let translations = &state.language.translations;

    let sites = if state.site_bookmarks.is_empty() {
        translations.no_sites.to_string()
    } else {
        state.site_bookmarks
            .iter()
            .map(|bookmark| {
                let configure_icon = hyper8_icons::configure(Some(translations.configure));
                let path = bookmark.path.to_string_lossy();
                format!(r#"
                    <div class="site" data-site-path="{path}">
                        <div class="pane" tabindex="0">
                            <span>{path}</span>
                            <button class="configure">{configure_icon}</button>
                        </div>
                        <div>
                            <button class="delete">Remove from list</button>
                        </div>
                    </div>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let t_add_site = translations.add_site;

    let languages = AVAILABLE_LANGUAGES
        .iter()
        .map(|language| {
            let code = language.code;
            let label = language.name;

            let selected = if code == state.language.code {
                "selected"
            } else {
                ""
            };

            format!(r#"<option {selected} value="{code}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("");


    format!(r#"
        <header>
            <form class="language">
                <select>
                    {languages}
                </select>
            </form>
        </header>
        <main>
            <div class="sites">
                {sites}
            </div>
            <div>
                <button class="add_site">{t_add_site}</button>
            </div>
        </main>
        <footer>
            <div>
                <a href="https://simonrepp.com/hyper8" target="_blank">
                    Website
                </a>
                <a href="https://simonrepp.com/hyper8/docs" target="_blank">
                    Docs
                </a>
                <a href="https://codeberg.org/simonrepp/hyper8/issues" target="_blank">
                    Report a bug
                </a>
                <a href="https://liberapay.com/simonrepp" target="_blank">
                    Liberapay
                </a>
                <a href="https://ko-fi.com/simonrepp" target="_blank">
                    Ko-fi
                </a>
            </div>
            <div>
                <a href="https://simonrepp.com/hyper8/changes/{VERSION_WITH_PATCH}" target="_blank">
                    {VERSION_ADAPTIVE_PATCH}
                </a>
            </div>
        </footer>
    "#)
}

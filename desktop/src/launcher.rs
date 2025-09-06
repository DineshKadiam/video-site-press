// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::AVAILABLE_LANGUAGES;

use crate::LauncherState;

pub fn render(state: &LauncherState) -> String {
    let translations = &state.language.translations;

    let sites = if state.config.sites.is_empty() {
        translations.no_sites.to_string()
    } else {
        state.config.sites
            .iter()
            .map(|site| {
                let path = site.to_string_lossy();
                format!(r#"
                    <div>
                        <span>{path}</span>
                        <button class="start" data-path="{path}">Start</button>
                        <button class="delete" data-path="{path}">Delete</button>
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

            let selected = if code == state.config.language_code {
                "selected"
            } else {
                ""
            };

            format!(r#"<option {selected} value="{code}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("");


    format!(r#"
        <main>
            <h1>Hyper 8</h1>
            <div>
                {sites}
            </div>
            <div>
                <button class="add_site">{t_add_site}</button>

                <form class="language">
                    <select>
                        {languages}
                    </select>
                </form>
            </div>
            <div>
                <a href="https://simonrepp.com/hyper8" target="_blank">
                    Hyper 8
                </a>
            </div>
        </main>
    "#)
}

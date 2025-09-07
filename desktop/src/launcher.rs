// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_core::AVAILABLE_LANGUAGES;
use hyper8_core::util::html_escape_outside_attribute;
use hyper8_version::{VERSION_ADAPTIVE_PATCH, VERSION_WITH_PATCH};

use crate::LauncherState;

pub fn render(state: &LauncherState) -> String {
    let translations = &state.language.translations;

    let logo_monochrome = hyper8_icons::logo_monochrome(None);

    let sites = if state.site_bookmarks.is_empty() {
        translations.no_sites.to_string()
    } else {
        let configure_icon = hyper8_icons::configure(Some(translations.configure));

        state.site_bookmarks
            .iter()
            .map(|bookmark| {
                let not_found;
                let label;

                if bookmark.found {
                    if let Some(title) = &bookmark.title {
                        label = html_escape_outside_attribute(title);
                    } else if let Some(live_url) = &bookmark.live_url {
                        label = live_url.to_string();
                    } else {
                        label = bookmark.site_dir.file_name().unwrap().to_string_lossy().to_string();
                    }

                    not_found = "";
                } else {
                    label = String::from("Not found");
                    not_found = "not_found";
                };

                let site_dir = bookmark.site_dir.to_string_lossy();

                formatdoc!(r#"
                    <li class="{not_found} site" data-site-dir="{site_dir}">
                        <div class="pane" tabindex="0">
                            <span class="labels">
                                {logo_monochrome}
                                <span>{site_dir}</span>
                            </span>
                            <span>
                                <span>{label}</span>
                                <button class="configure">{configure_icon}</button>
                            </span>
                        </div>
                        <div>
                            <button class="delete">Remove from list</button>
                            <button class="open_in_file_explorer">Open in file explorer</button>
                        </div>
                    </li>
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

    let close_icon = hyper8_icons::failure(translations.close);
    let link_icon = hyper8_icons::link(Some(translations.external_link));

    format!(r#"
        <header data-tauri-drag-region>
            <img alt="Hyper 8" src="assets/logo.svg">
            <button class="close">{close_icon}</button>
            <a class="version" href="https://simonrepp.com/hyper8/changes/{VERSION_WITH_PATCH}" target="_blank">
                {link_icon} {VERSION_ADAPTIVE_PATCH}
            </a>
        </header>
        <main>
            <ul class="sites">
                <li>
                    <button class="add_site">{logo_monochrome} {t_add_site}</button>
                    /
                    <button class="quick_start">{logo_monochrome} Quick start new site</button>
                </li>
                {sites}
            </ul>
        </main>
        <footer>
            <div>
                <a href="https://simonrepp.com/hyper8" target="_blank">
                    {link_icon} Website
                </a>
                <a href="https://simonrepp.com/hyper8/#donate" target="_blank">
                    {link_icon} Donate
                </a>
            </div>
            <div>
                <form class="language">
                    <select>
                        {languages}
                    </select>
                </form>
            </div>
        </footer>
    "#)
}

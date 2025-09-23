// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;
use std::sync::{Arc, Mutex};

use tauri::State;

use hyper8_core::Theme;
use hyper8_core::AVAILABLE_LANGUAGES;
use hyper8_core::util::html_escape_outside_attribute;
use hyper8_version::{VERSION_ADAPTIVE_PATCH, VERSION_WITH_PATCH};

use crate::{EditorMode, LauncherState};

#[tauri::command]
pub fn render(
    state: State<'_, Arc<Mutex<LauncherState>>>
) -> String {
    let launcher_state = state.lock().unwrap();

    let language_code;
    let theme;
    let translations;

    {
        let editor_state = launcher_state.editor_state.lock().unwrap();
        language_code = editor_state.language.code.clone();
        theme = editor_state.theme.clone();
        translations = editor_state.language.translations.clone();
    }

    let create_icon = hyper8_icons::CREATE;
    let t_create_new_site = translations.create_new_site;
    let create_new_site = formatdoc!(r#"
        <button class="create_new_site">
            {create_icon}
            <span>{t_create_new_site}</span>
        </button>
    "#);

    let browse_icon = hyper8_icons::BROWSE;

    let t_open_existing_site = translations.open_existing_site;
    let open_existing_site = formatdoc!(r#"
        <button class="open_existing_site">
            {browse_icon}
            <span>{t_open_existing_site}</span>
        </button>
    "#);

    let sites_and_actions = if launcher_state.site_bookmarks.is_empty() {
        formatdoc!(r#"
            {create_new_site}
            {open_existing_site}
        "#)
    } else {
        let configure_icon = hyper8_icons::configure(Some(translations.configure));

        let sites = launcher_state.site_bookmarks
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
                    label = translations.not_found.to_string();
                    not_found = "not_found";
                };

                let site_dir = bookmark.site_dir.to_string_lossy();

                let t_remove_from_list = translations.remove_from_list;
                let t_open_in_file_manager = translations.open_in_file_manager;
                formatdoc!(r#"
                    <li class="{not_found} site" data-site-dir="{site_dir}">
                        <div class="pane" tabindex="0">
                            {browse_icon}
                            <span>{site_dir}</span>
                            <span>{label}</span>
                            <button class="configure">{configure_icon}</button>
                        </div>
                        <div>
                            <button class="remove_site_bookmark">{t_remove_from_list}</button>
                            <button class="open_in_file_manager">{t_open_in_file_manager}</button>
                        </div>
                    </li>
                "#)
            })
            .collect::<Vec<String>>()
            .join("");

        formatdoc!(r#"
            <ul class="sites">
                {sites}
            </ul>
            {open_existing_site}
            {create_new_site}
        "#)
    };

    let language = AVAILABLE_LANGUAGES
        .iter()
        .find_map(|language| {
            if language.code == language_code {
                Some(language.name)
            } else {
                None
            }
        })
        .unwrap();

    let languages = AVAILABLE_LANGUAGES
        .iter()
        .map(|language| {
            let code = language.code;
            let label = language.name;

            formatdoc!(r#"
                <button data-code="{code}">
                    <span>{label}</span>
                </button>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let mode = launcher_state.editor_mode.label(&translations);
    let mode_icon = launcher_state.editor_mode.icon();
    let modes = [EditorMode::Collaborative, EditorMode::BrowserBased, EditorMode::Standalone]
        .iter()
        .map(|mode| {
            let description = mode.description(&translations);
            let icon = mode.icon();
            let key = mode.key();
            let label = mode.label(&translations);

            formatdoc!(r#"
                <button data-key="{key}">
                    <span>{icon} {label}</span>
                    <span>{description}</span>
                </button>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let theme_icon = theme.icon();
    let themes = Theme::variants(&translations)
        .iter()
        .map(|(theme, label)| {
            let key = theme.key();
            let icon = theme.icon();

            formatdoc!(r#"
                <button data-key="{key}">
                    <span>{icon} {label}</span>
                </button>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let close_icon = hyper8_icons::failure(translations.close);
    let language_icon = hyper8_icons::LANGUAGE;
    let link_icon = hyper8_icons::link(Some(translations.external_link));

    let t_donate = translations.donate;
    let t_website = translations.website;
    format!(r#"
        <header data-tauri-drag-region>
            <img alt="Hyper 8" src="assets/logo.svg">
            <button class="close">{close_icon}</button>
            <a class="version" href="https://simonrepp.com/hyper8/changes/{VERSION_WITH_PATCH}" target="_blank">
                {link_icon} {VERSION_ADAPTIVE_PATCH}
            </a>
        </header>
        <main>
            {sites_and_actions}
        </main>
        <footer>
            <div>
                <a href="https://simonrepp.com/hyper8" target="_blank">
                    {link_icon} {t_website}
                </a>
                <a href="https://simonrepp.com/hyper8/#donate" target="_blank">
                    {link_icon} {t_donate}
                </a>
            </div>
            <div class="options">
                <button class="mode">
                    {mode_icon} {mode}
                </button>
                <div class="selector">
                    <div class="panel">
                        {modes}
                    </div>
                </div>
                <button class="language">
                    {language_icon} {language}
                </button>
                <div class="selector language">
                    <div class="panel">
                        {languages}
                    </div>
                </div>
                <button class="theme">
                    {theme_icon}
                </button>
                <div class="selector theme">
                    <div class="panel">
                        {themes}
                    </div>
                </div>
            </div>
        </footer>
        <dialog class="overlay" data-tauri-drag-region></dialog>
    "#)
}
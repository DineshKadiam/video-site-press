// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;


use indoc::formatdoc;
use tauri::{
    AppHandle,
    Emitter,
    Manager,
    State,
    WebviewWindow
};
use tauri_plugin_dialog::{
    DialogExt,
    FilePath,
    MessageDialogButtons,
    MessageDialogKind
};

use hyper8_core::{Language, Theme};
use hyper8_core::{GENERIC_DARK_THEME_VARS, GENERIC_LIGHT_THEME_VARS};

use crate::{LauncherState, SiteBookmark};
use crate::EXIT_CODE_SUCCESS;
use crate::editor;
use crate::launcher;

#[tauri::command]
pub fn change_language(
    code: &str,
    state: State<'_, Mutex<LauncherState>>
) {
    let language = Language::from_code(code);

    let mut state_mut = state.lock().unwrap();
    state_mut.language = language;
    state_mut.persist();
}

#[tauri::command]
pub fn close(app: AppHandle) {
    app.exit(EXIT_CODE_SUCCESS);
}

#[tauri::command]
pub fn create_new_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Mutex<LauncherState>>
) {
    let state = state.lock().unwrap();

    let translations = &state.language.translations;

    // TODO: Handle the None case as well (e.g. by not offering Quick start in
    // the launcher to begin with)
    if let Some(home_dir) = dirs::home_dir() {
        let t_new_hyper8_site_folder_name = translations.new_hyper8_site_folder_name;
        let mut site_dir = home_dir.join(t_new_hyper8_site_folder_name);

        let mut duplicate = 2;
        while site_dir.exists() {
            site_dir = home_dir.join(format!("{t_new_hyper8_site_folder_name}({duplicate})"));
            duplicate += 1;
        }

        let site_dir_display = site_dir.display();
        let t_create_new_site_confirmation_notes = translations.create_new_site_confirmation_notes;
        let t_this_will_create_an_empty_site_folder_in_your_home_directory = translations.this_will_create_an_empty_site_folder_in_your_home_directory;
        let message = formatdoc!("
            {t_this_will_create_an_empty_site_folder_in_your_home_directory}

            {site_dir_display}

            {t_create_new_site_confirmation_notes}
        ");

        let t_cancel = translations.cancel;
        let t_create_new_site = translations.create_new_site;
        let t_create_site = translations.create_site;

        app
            .dialog()
            .message(message)
            .kind(MessageDialogKind::Info)
            .title(t_create_new_site)
            .buttons(
                MessageDialogButtons::OkCancelCustom(
                    t_create_site.to_string(),
                    t_cancel.to_string()
                )
            )
            .show(move |confirmed| {
                if confirmed {
                    fs::create_dir(&site_dir).unwrap();

                    let bookmark = SiteBookmark::new(&site_dir);

                    // Note that the outer state handle we unlock in the root
                    // scope of delete_site(…) immediately falls out of scope.
                    // Here we are in a new, later occuring execution context,
                    // hence we obtain a new handle on state.
                    let state = app.state::<Mutex<LauncherState>>();

                    let translations = {
                        let mut state_mut = state.lock().unwrap();

                        state_mut.site_bookmarks.insert(0, bookmark);
                        state_mut.persist();

                        state_mut.language.translations.clone()
                    };

                    editor::start_editor(
                        app,
                        launcher_window,
                        &site_dir,
                        translations
                    );

                }
            });
    }
}

#[tauri::command]
pub fn get_language(
    state: State<'_, Mutex<LauncherState>>
) -> String {
    let state = state.lock().unwrap();
    state.language.code.to_string()
}

#[tauri::command]
pub fn get_theme(prefer_dark: bool) -> String {
    let vars = if prefer_dark { GENERIC_DARK_THEME_VARS } else { GENERIC_LIGHT_THEME_VARS };

    let vars_hsl = vars.print_hsl();
    let vars_oklch = vars.print_oklch();

    let splash_vars = Theme::print_splash_vars();

    formatdoc!(r#"
        {splash_vars}
        :root {{
            {vars_hsl}
        }}
        @supports (color: oklch(0% 0 0)) {{
            :root {{
                {vars_oklch}
            }}
        }}
    "#)
}

#[tauri::command]
pub fn open_bookmarked_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    site_dir: &Path,
    state: State<'_, Mutex<LauncherState>>
) {
    let translations = {
        let mut state_mut = state.lock().unwrap();

        let index = state_mut.site_bookmarks
            .iter()
            .position(|bookmark| bookmark.site_dir == site_dir)
            .unwrap();

        // If the site is not at the top of our list of bookmarks,
        // put it there.
        if index > 0 {
            let bookmark = state_mut.site_bookmarks.remove(index);
            state_mut.site_bookmarks.insert(0, bookmark);
            state_mut.persist();
        }

        state_mut.language.translations.clone()
    };

    editor::start_editor(
        app,
        launcher_window,
        site_dir,
        translations
    );
}

#[tauri::command]
pub fn open_existing_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Mutex<LauncherState>>
) {
    let state = state.lock().unwrap();

    app
        .dialog()
        .file()
        .set_title(state.language.translations.pick_an_empty_or_preexisting_site_directory)
        .pick_folder(move |result| {
            if let Some(FilePath::Path(site_dir)) = result {
                let bookmark = SiteBookmark::new(&site_dir);

                let state = app.state::<Mutex<LauncherState>>();

                let translations = {
                    let mut state_mut = state.lock().unwrap();

                    // If an already bookmarked site is opened, remove it form
                    // the bookmarks before we add it anew
                    if let Some(index) = state_mut.site_bookmarks
                        .iter()
                        .position(|bookmark| bookmark.site_dir == site_dir) {
                        state_mut.site_bookmarks.remove(index);
                    }

                    state_mut.site_bookmarks.insert(0, bookmark);

                    state_mut.persist();

                    state_mut.language.translations.clone()
                };

                let _ = app.emit("update-body", ());

                editor::start_editor(
                    app,
                    launcher_window,
                    &site_dir,
                    translations
                );
            }
        });
}

#[tauri::command]
pub fn open_in_file_manager(site_dir: &Path) {
    let _ = tauri_plugin_opener::open_path(site_dir, None::<&str>);
}

#[tauri::command]
pub fn remove_site_bookmark(
    app: AppHandle,
    site_dir: PathBuf,
    state: State<'_, Mutex<LauncherState>>
) {
    let state = state.lock().unwrap();

    let translations = &state.language.translations;

    let t_cancel = translations.cancel;
    let t_delete = translations.delete;
    let t_really_delete = translations.really_delete;

    app
        .dialog()
        .message(t_really_delete)
        .kind(MessageDialogKind::Warning)
        .title(t_really_delete)
        .buttons(
            MessageDialogButtons::OkCancelCustom(
                t_delete.to_string(),
                t_cancel.to_string()
            )
        )
        .show(move |confirmed| {
            if confirmed {
                // Note that the outer state handle we unlock in the root
                // scope of remove_site_bookmark(…) immediately falls out of scope.
                // Here we are in a new, later occuring execution context,
                // hence we obtain a new handle on state.
                let state = app.state::<Mutex<LauncherState>>();

                {
                    let mut state = state.lock().unwrap();

                    if let Some(index) = state.site_bookmarks
                        .iter()
                        .position(|bookmark| bookmark.site_dir == site_dir) {
                        state.site_bookmarks.swap_remove(index);
                    }

                    state.persist();
                }

                let _ = app.emit("update-body", ());
            }
        });
}

#[tauri::command]
pub fn render(
    state: State<'_, Mutex<LauncherState>>
) -> String {
    let state = state.lock().unwrap();
    launcher::render(&state)
}

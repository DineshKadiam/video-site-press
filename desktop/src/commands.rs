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
pub async fn add_site(app: AppHandle) {
    app
        .dialog()
        .file()
        .set_title("Pick an (empty or pre-existing) site directory")
        .pick_folder(move |result| {
            if let Some(FilePath::Path(site_dir)) = result {
                let site_bookmark = SiteBookmark::new(site_dir);

                let state = app.state::<Mutex<LauncherState>>();

                {
                    let mut state_mut = state.lock().unwrap();
                    state_mut.site_bookmarks.push(site_bookmark);
                    state_mut.persist();
                }

                let _ = app.emit("update-body", ());
            }
        });
}

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
pub fn delete_site(
    app: AppHandle,
    site_dir: PathBuf,
    state: State<'_, Mutex<LauncherState>>
) {
    let state = state.lock().unwrap();

    let t_cancel = state.language.translations.cancel;
    let t_delete = state.language.translations.delete;
    let t_really_delete = state.language.translations.really_delete;

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
                // scope of delete_site(…) immediately falls out of scope.
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
pub fn open_in_file_explorer(site_dir: &Path) {
    let _ = tauri_plugin_opener::open_path(site_dir, None::<&str>);
}

#[tauri::command]
pub fn quick_start(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Mutex<LauncherState>>
) {
    let state = state.lock().unwrap();

    let t_cancel = state.language.translations.cancel;

    // TODO: Handle the None case as well (e.g. by not offering Quick start in
    // the launcher to begin with)
    if let Some(home_dir) = dirs::home_dir() {
        let mut site_dir = home_dir.join("hyper8-quick-start-site");

        let mut duplicate = 2;
        while site_dir.exists() {
            site_dir = home_dir.join(format!("hyper8-quick-start-site-{duplicate}"));
            duplicate += 1;
        }

        let message = formatdoc!("
            This will create the empty site directory '{}' and open it in the editor right away.

            You will be able to rename and relocate this directory at any future point.
            You can also delete the directory at any point, no other traces will be left.

            Proceed?",
            site_dir.display()
        );

        app
            .dialog()
            .message(message)
            .kind(MessageDialogKind::Info)
            .title("Quick start")
            .buttons(
                MessageDialogButtons::OkCancelCustom(
                    String::from("Create and open"),
                    t_cancel.to_string()
                )
            )
            .show(move |confirmed| {
                if confirmed {
                    fs::create_dir(&site_dir).unwrap();

                    let site_bookmark = SiteBookmark::new(site_dir.clone());

                    // Note that the outer state handle we unlock in the root
                    // scope of delete_site(…) immediately falls out of scope.
                    // Here we are in a new, later occuring execution context,
                    // hence we obtain a new handle on state.
                    let state = app.state::<Mutex<LauncherState>>();

                    let translations = {
                        let mut state_mut = state.lock().unwrap();

                        state_mut.site_bookmarks.push(site_bookmark);
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
pub fn render(
    state: State<'_, Mutex<LauncherState>>
) -> String {
    let state = state.lock().unwrap();
    launcher::render(&state)
}

#[tauri::command]
pub fn start_editor(
    app: AppHandle,
    launcher_window: WebviewWindow,
    site_dir: &Path,
    state: State<'_, Mutex<LauncherState>>
) {
    let translations = {
        let state = state.lock().unwrap();
        state.language.translations.clone()
    };

    editor::start_editor(
        app,
        launcher_window,
        site_dir,
        translations
    );
}

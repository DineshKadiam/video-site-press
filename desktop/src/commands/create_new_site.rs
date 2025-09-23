// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::{Arc, Mutex};

use indoc::formatdoc;
use tauri::{
    AppHandle,
    Manager,
    State,
    WebviewWindow
};
use tauri_plugin_dialog::{
    DialogExt,
    MessageDialogButtons,
    MessageDialogKind
};

use crate::editor;
use crate::{LauncherState, SiteBookmark};

#[tauri::command]
pub fn create_new_site(
    app: AppHandle,
    launcher_window: WebviewWindow,
    state: State<'_, Arc<Mutex<LauncherState>>>
) {
    let launcher_state = state.lock().unwrap();

    let translations = launcher_state.get_translations();

    // TODO: Handle the None case by finding some other usable dir.
    //       (question is which though?)
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
                    // scope of create_new_site(…) immediately falls out of scope.
                    // Here we are in a new, later occuring execution context,
                    // hence we obtain a new handle on state.
                    let state = app.state::<Arc<Mutex<LauncherState>>>();

                    let editor_mode;
                    let translations;

                    {
                        let mut launcher_state = state.lock().unwrap();

                        launcher_state.site_bookmarks.insert(0, bookmark);
                        launcher_state.persist();

                        editor_mode = launcher_state.editor_mode;
                        translations = launcher_state.get_translations();
                    };

                    editor::start_editor(
                        app,
                        launcher_window,
                        editor_mode,
                        &site_dir,
                        translations
                    );
                }
            });
    }
}

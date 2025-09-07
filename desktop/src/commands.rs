// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use indoc::formatdoc;
use tauri::{
    AppHandle,
    Emitter,
    Manager,
    State,
    Url,
    WebviewUrl,
    WebviewWindow,
    WebviewWindowBuilder,
    WindowEvent
};
use tauri_plugin_dialog::{
    DialogExt,
    FilePath,
    MessageDialogButtons
};

use hyper8_core::{
    Cache,
    Context,
    Language,
    Site
};
use hyper8_core::start_workers;
use hyper8_core::{GENERIC_DARK_THEME_VARS, GENERIC_LIGHT_THEME_VARS};
use hyper8_core::watcher::watch_site_dir;

use crate::launcher;

use crate::{LauncherState, SiteBookmark};

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

    formatdoc!(r#"
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

    if !site_dir.exists() {
        eprintln!(
            "Site directory {site_dir:?} does not exist, please create it manually (just an empty folder).",
        );
        return;
    }

    // This compiles to fs::canonicalize(...) on all platforms but windows,
    // on windows it instead tries to supply a canonicalized path that uses
    // the regular form (e.g. "C:\\Foo") instead of the UNC form ("\\?C:\\Foo")
    // when possible (for background see https://crates.io/crates/dunce).
    let site_dir = dunce::canonicalize(site_dir).unwrap();

    let build_dir = site_dir.join(".hyper8_build");
    let cache_dir = site_dir.join(".hyper8_cache");

    let cache = Mutex::new(Cache::retrieve(&cache_dir));

    let context = Context::new(
        build_dir,
        cache,
        cache_dir,
        Arc::new(Site::new()),
        site_dir.clone()
    );

    // TODO: Would be great if we could somehow resolve the conundrum that
    //       context depends on site and site depends on context, which
    //       forces us to initialize context with a dummy site and replace it here.
    context.replace_site(Site::read_dir(&context, &site_dir));

    let context = Arc::new(context);

    context.compute_job_queue();
    start_workers(&context, None, &translations);

    watch_site_dir(Arc::clone(&context));

    launcher_window.hide().unwrap();

    // TODO: This url must be returned to us from hyper8_web::start_server somehow,
    //       and additionally only after the server's .run() method was invoked
    //       (tokio::wait!(...) construct), and then we also need to run this in
    //       yet another thread, because this one will be busy waiting for the start_server(...)
    //       call to return (which it only does once we kill the process)
    let webview_url = WebviewUrl::External(Url::parse("http://127.0.0.1:8080").unwrap());

    let window = WebviewWindowBuilder::new(&app, "hyper8-web", webview_url)
        .title(&format!("Hyper 8 – {}", site_dir.display()))
        .build()
        .unwrap();

    window.on_window_event(move |event: &WindowEvent| {
        if let WindowEvent::Destroyed = event {
            app.exit(0);
        }
    });

    let _ = window.maximize();

    thread::spawn(|| {
        // TODO: Currently also opens the editor in the default browser, but shouldn't.
        //       (baked in functionality for hyper8-cli right now)
        let result = hyper8_web::start_server(context, None, None);

        if !result.is_ok() {
            // TODO: Possibly do something with this state
        }
    });
}

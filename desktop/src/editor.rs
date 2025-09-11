// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::sync::{Arc, Mutex};

use indoc::formatdoc;
use tauri::{
    AppHandle,
    Emitter,
    Manager,
    Url,
    WebviewUrl,
    WebviewWindow,
    WebviewWindowBuilder,
    WindowEvent
};

use hyper8_core::{
    Cache,
    Context,
    Site
};
use hyper8_core::start_workers;
use hyper8_core::watcher::watch_site_dir;
use hyper8_translations::Translations;
use hyper8_web::{EditorState, Mode};

use crate::EXIT_CODE_SUCCESS;

pub fn start_editor(
    app: AppHandle,
    launcher_window: WebviewWindow,
    mode: Mode,
    site_dir: &Path,
    translations: Translations,
) {
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

    let ip = if mode == Mode::Collaborative {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    } else {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    };

    let editor_state = (*app.state::<Arc<Mutex<EditorState>>>()).clone();

    let result = hyper8_web::start_server(
        context,
        editor_state,
        Some(ip),
        None
    );

    match result {
        Ok(url) => {
            if mode == Mode::Standalone {
                launcher_window.hide().unwrap();

                let webview_url = WebviewUrl::External(Url::parse(&url).unwrap());

                let window = WebviewWindowBuilder::new(&app, "editor", webview_url)
                    .title(&format!("Hyper 8 – {}", site_dir.display()))
                    .build()
                    .unwrap();

                window.on_window_event(move |event: &WindowEvent| {
                    if let WindowEvent::Destroyed = event {
                        app.exit(EXIT_CODE_SUCCESS);
                    }
                });

                let _ = window.maximize();
            } else {
                let t_shut_down_editor_and_quit = translations.shut_down_editor_and_quit;
                let t_the_hyper8_editor_is_available_at_xxx = translations
                    .the_hyper8_editor_is_available_at_xxx(&url);

                let content = formatdoc!("
                    <div>{t_the_hyper8_editor_is_available_at_xxx}</div>
                    <button>{t_shut_down_editor_and_quit}</button>
                ");

                let _ = app.emit("show-overlay", content);

                let _ = tauri_plugin_opener::open_url(url, None::<&str>);
            };
        }
        Err(err) => {
            // TODO: Provide the error message in the application, then everything can crash.
            panic!("{err}");
        }
    }
}

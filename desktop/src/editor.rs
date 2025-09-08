// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::{
    AppHandle,
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

use crate::EXIT_CODE_SUCCESS;

pub fn start_editor(
    app: AppHandle,
    launcher_window: WebviewWindow,
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
            app.exit(EXIT_CODE_SUCCESS);
        }
    });

    let _ = window.maximize();

    thread::spawn(|| {
        let result = hyper8_web::start_server(context, None, false, None);

        if !result.is_ok() {
            // TODO: Possibly do something with this state
        }
    });
}
// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use sys_locale::get_locale;

use hyper8_core::{
    Cache,
    Context,
    Language,
    Site
};
use hyper8_core::{perform_build, start_preview_server, start_workers};
use hyper8_core::watcher::watch_site_dir;
use hyper8_web::EditorState;

mod args;
mod deploy;

use deploy::perform_deploy_blocking_cli;

fn main() -> ExitCode {
    env_logger::init();

    let language = get_locale()
        .map(|locale| Language::from_code(&locale))
        .unwrap_or_else(|| Language::default());

    let translations = &language.translations;

    let args = args::parse();

    let site_dir = args.get_one::<PathBuf>("site-dir").unwrap();

    if !site_dir.exists() {
        eprintln!(
            "Site directory {site_dir:?} does not exist, please create it manually (just an empty folder).",
        );
        return ExitCode::FAILURE;
    }

    // This compiles to fs::canonicalize(...) on all platforms but windows,
    // on windows it instead tries to supply a canonicalized path that uses
    // the regular form (e.g. "C:\\Foo") instead of the UNC form ("\\?C:\\Foo")
    // when possible (for background see https://crates.io/crates/dunce).
    let site_dir = dunce::canonicalize(site_dir).unwrap();

    let build_dir = args.get_one::<PathBuf>("build-dir")
        .as_ref()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| site_dir.join(".hyper8_build"));

    let cache_dir = args.get_one::<PathBuf>("cache-dir")
        .as_ref()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| site_dir.join(".hyper8_cache"));

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

    let workers = args.get_one::<usize>("workers").map(|count| *count);

    context.compute_job_queue();
    start_workers(&context, workers, &translations);

    let ip = args.get_one::<IpAddr>("ip");
    let port = args.get_one::<u16>("port");

    if args.get_flag("build") {
        if let Err(err) = context.await_job_queue() {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }

        perform_build(&context);

        if args.get_flag("preview") {
            let site = context.get_site();

            if site.clean_urls {
                // Here we serve the preview through an actual http server so
                // that /foo/ gets resolved to /foo/index.html.
                let result = start_preview_server(&context.build_dir, ip, port);

                if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
            } else {
                // We don't need an actively running server to preview a build
                // without clean urls, we can just open everything directly in
                // a browser.
                let local_file_url = context.build_dir.join("index.html");
                if webbrowser::open(&local_file_url.to_string_lossy()).is_err() {
                    eprintln!("Could not open browser for previewing the site");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        } else if args.get_flag("deploy") {
            let site = context.get_site();

            match perform_deploy_blocking_cli(&context, &site, &translations) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
            }
        } else {
            ExitCode::SUCCESS
        }
    } else {
        println!("Using site directory: {}", context.site_dir.display());

        watch_site_dir(Arc::clone(&context));

        let editor_state = Arc::new(Mutex::new(EditorState::init()));

        let result = hyper8_web::start_server(
            context,
            editor_state,
            ip.copied(),
            port.copied()
        );

        match result {
            Ok(url) => {
                println!("Serving the editor at: {url}");

                match webbrowser::open(&url) {
                    Ok(()) => println!("The editor has been opened in your default browser."),
                    Err(err) => eprintln!("Please open the url manually in your browser (an automatic attempt to do so failed with the error '{err}').")
                }

                println!(r#"Enter "exit" and confirm with Enter or press Ctrl+C to shut down the editor (if you have pending encoding jobs they will be discarded, otherwise this is safe to do at any point)"#);

                // TODO: We could offer a simple utility console here (to get debug
                // information, list jobs, etc.).
                let mut input = String::new();
                loop {
                    if io::stdin().read_line(&mut input).is_ok() {
                        if input.trim() == "exit" {
                            break;
                        }
                    }
                }

                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("The server could not be started ({err})");
                ExitCode::FAILURE
            }
        }
    }
}

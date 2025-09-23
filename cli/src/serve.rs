// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::net::IpAddr;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use clap::ArgMatches;

use hyper8_core::Context;
use hyper8_core::watcher::watch_site_dir;
use hyper8_web::EditorState;

pub fn serve(
    args: ArgMatches,
    context: Arc<Context>
) -> ExitCode {
    println!("Using site directory: {}", context.site_dir.display());

    watch_site_dir(Arc::clone(&context));

    let editor_state = Arc::new(Mutex::new(EditorState::init()));

    let ip = args.get_one::<IpAddr>("ip");
    let port = args.get_one::<u16>("port");

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

            println!(r#"Enter "q", "quit" or "exit" and confirm with Enter to shut down the editor (if you have pending encoding jobs they will be discarded, otherwise this is safe to do at any point)"#);

            // TODO: We could offer a simple utility console here (to get debug
            // information, list jobs, etc.).
            let mut input = String::new();
            loop {
                if io::stdin().read_line(&mut input).is_ok() {
                    match input.trim() {
                        "exit" |
                        "q" |
                        "quit" => break,
                        _ => input.clear()
                    }
                } else {
                    break;
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

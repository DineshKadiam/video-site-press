// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::mpsc::channel;

use clap::ArgMatches;

use hyper8_core::Context;
use hyper8_core::watcher::watch_site_dir;
use hyper8_web::{EditorState, ServerMode};

pub fn serve(
    args: ArgMatches,
    context: Arc<Context>
) -> ExitCode {
    println!("Using site directory: {}", context.site_dir.display());

    watch_site_dir(Arc::clone(&context));

    let inherited_editor_state = EditorState::retrieve_or_initialize();
    let server_mode = ServerMode::MultipleRemoteUsers { inherited_editor_state };

    let ip = args.get_one::<IpAddr>("ip");
    let port = args.get_one::<u16>("port");

    let result = hyper8_web::start_server(
        context,
        ip.copied(),
        port.copied(),
        server_mode
    );

    match result {
        Ok(url) => {
            println!("Serving the editor at: {url}");

            match webbrowser::open(&url) {
                Ok(()) => println!("The editor has been opened in your default browser."),
                Err(err) => eprintln!("Please open the url manually in your browser (an automatic attempt to do so failed with the error '{err}').")
            }

            println!(r#"Press Ctrl+C to shut down the editor (if you have pending encoding jobs they will be discarded, otherwise this is safe to do at any point)"#);

            // Wait for Ctrl+C using the ctrlc crate and channel-based signaling
            let (signal_sender, signal_receiver) = channel();
            ctrlc::set_handler(move || { signal_sender.send(()).unwrap(); }).unwrap();
            signal_receiver.recv().unwrap();

            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("The server could not be started ({err})");
            ExitCode::FAILURE
        }
    }
}

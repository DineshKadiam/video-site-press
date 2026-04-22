// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use actix_files::Files;
use actix_web::{App, HttpServer};
use indoc::formatdoc;

use crate::DEFAULT_IP;
use crate::{default_ports_display, default_ports_iter};

#[actix_web::main]
pub async fn start(
    build_dir: &Path,
    ip_requested: Option<&IpAddr>,
    port_requested: Option<&u16>
) -> Result<(), ()> {
    let bind_server = |build_dir_moving: PathBuf, ip: IpAddr, port: u16| {
        HttpServer::new(move || {
            App::new()
                .service(
                    // Static file serving of built site
                    Files::new("/", &build_dir_moving)
                        .redirect_to_slash_directory()
                        .index_file("index.html")
                )
        })
        .bind((ip, port))
    };

    let ip = ip_requested.unwrap_or(&DEFAULT_IP);

    let (server, port_bound) = if let Some(port) = port_requested {
        match bind_server(build_dir.to_path_buf(), *ip, *port) {
            Ok(server) => (server, *port),
            Err(err) => {
                eprintln!("Could not bind preview server to {ip}:{port} ({err})");
                return Err(());
            }
        }
    } else {
        let mut default_ports = default_ports_iter().peekable();

        loop {
            let port = default_ports.next().unwrap();

            match bind_server(build_dir.to_path_buf(), *ip, port) {
                Ok(server) => break (server, port),
                Err(err) => {
                    if default_ports.peek().is_none() {
                        let default_ports_display = default_ports_display();

                        let message = formatdoc!("
                            Could not bind preview server to {ip} on any of the default ports ({default_ports_display}).
                            The last error was: {err}
                            Inspect your system and ensure that at least one of the default ports is free to use by Hyper 8.
                            When running from the command line you can use --port to manually set a port to work around this as well.
                        ");

                        eprintln!("{message}");

                        return Err(());
                    }
                }
            }
        }
    };

    let url = format!("http://{ip}:{port_bound}");

    println!("Serving the preview at {url} (open this address in your browser if it doesn't by itself)");
    println!("Press Ctrl+C to shut down the preview server (this is safe to do at any point)");

    let open_browser = || async {
        if webbrowser::open(&url).is_err() {
            eprintln!("Could not automatically open browser with the preview, please do so manually")
        }
    };

    tokio::join!(server.run(), open_browser()).0.unwrap();

    Ok(())
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};

use actix_files::Files;
use actix_web::{App, HttpServer};

const DEFAULT_PREVIEW_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_PREVIEW_PORT: u16 = 49737; // "Dial 127.0.0.1:HYPER"

/// When DEFAULT_PREVIEW_PORT is unavailable, we try DEFAULT_PREVIEW_PORT + 1,
/// then DEFAULT_PREVIEW_PORT + 2, etc., but after MAX_PORT_ATTEMPTS
/// iterations we stop, as probably something is else is wrong then, rather
/// than all ports taken.
const MAX_PORT_ATTEMPTS: u16 = 10;

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

    let ip = ip_requested.unwrap_or(&DEFAULT_PREVIEW_IP);

    let (server, port_bound) = if let Some(port) = port_requested {
        match bind_server(build_dir.to_path_buf(), *ip, *port) {
            Ok(server) => (server, *port),
            Err(err) => {
                eprintln!("Could not bind preview server to {ip}:{port} ({err})");
                return Err(());
            }
        }
    } else {
        let mut port = DEFAULT_PREVIEW_PORT;

        loop {
            match bind_server(build_dir.to_path_buf(), *ip, port) {
                Ok(server) => break (server, port),
                Err(err) => {
                    if port > DEFAULT_PREVIEW_PORT + MAX_PORT_ATTEMPTS {
                        eprintln!("Could not bind preview server to {ip} on any port in the range {DEFAULT_PREVIEW_PORT}-{port} (last error was {err}).\nUse --port to manually set a port and/or check that no other issue is at play");
                        return Err(());
                    }

                    port += 1;
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

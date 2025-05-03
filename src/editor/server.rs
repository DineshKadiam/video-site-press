// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use actix_files::Files;
use actix_web::{App, HttpServer};
use actix_web::web::{Data, get, post, resource};

use crate::Context;
use crate::editor::endpoints::{
    build,
    change_editor_config,
    change_tree_view_order,
    collection,
    convert,
    deploy,
    deployment,
    favicon,
    index,
    jobs,
    playlist,
    processing,
    scripts,
    splash_logo,
    styles,
    site,
    video
};

const DEFAULT_EDITOR_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_EDITOR_PORT: u16 = 8080;

/// When DEFAULT_EDITOR_PORT is unavailable, we try DEFAULT_EDITOR_PORT + 1,
/// then DEFAULT_EDITOR_PORT + 2, etc., but after MAX_PORT_ATTEMPTS
/// iterations we stop, as probably something is else is wrong then, rather
/// than all ports taken.
const MAX_PORT_ATTEMPTS: u16 = 10;

#[actix_web::main]
pub async fn start(
    context: Arc<Context>,
    ip_requested: &Option<IpAddr>,
    port_requested: &Option<u16>
) {
    let bind_server = |data_moving: Data<Arc<Context>>, ip: IpAddr, port: u16| {
        HttpServer::new(move || {
            App::new()
                .app_data(data_moving.clone())
                .route("/", get().to(index::index))
                .route("/build/{site_path:.*}", post().to(build))
                .route("/convert-to-collection/{site_path:.*}", post().to(convert::to_collection))
                .route("/convert-to-playlist/{site_path:.*}", post().to(convert::to_playlist))
                .route("/convert-to-video/{site_path:.*}", post().to(convert::to_video))
                .route("/change-editor-config", post().to(change_editor_config))
                .route("/change-tree-view-order", post().to(change_tree_view_order))
                .route("/collection/add-link/{site_path:.*}", post().to(collection::add_link))
                .route("/collection/create-playlist/{site_path:.*}", post().to(collection::create_playlist))
                .route("/collection/create-subcollection/{site_path:.*}", post().to(collection::create_subcollection))
                .route("/collection/create-video/{site_path:.*}", post().to(collection::create_video))
                .route("/collection/dangerous-actions/{site_path:.*}", get().to(collection::dangerous_actions))
                .route("/collection/delete/{site_path:.*}", post().to(collection::delete))
                .route("/collection/delete-banner/{site_path:.*}", post().to(collection::delete_banner))
                .route("/collection/delete-link/{site_path:.*}", post().to(collection::delete_link))
                .service(
                    resource("/collection/edit/{site_path:.*}")
                        .route(get().to(collection::edit))
                        .route(post().to(collection::update))
                )
                .service(
                    resource("/collection/manifest/{site_path:.*}")
                        .route(get().to(collection::edit_manifest))
                        .route(post().to(collection::update_manifest))
                )
                .route("/collection/move/{site_path:.*}", post().to(collection::r#move))
                .route("/collection/update-banner/{site_path:.*}", post().to(collection::update_banner))
                .route("/collection/upload-banner/{site_path:.*}", post().to(collection::upload_banner))
                .route("/deploy", post().to(deploy::deploy))
                .route("/deployment", get().to(deployment))
                .route("/favicon.png", get().to(favicon))
                // .route("/formats", post().to(settings::formats))
                .route("/jobs", get().to(jobs))
                // .route("/playlist", post().to(playlist::create))
                .route("/playlist/add-link/{site_path:.*}", post().to(playlist::add_link))
                .route("/playlist/create-video/{site_path:.*}", post().to(playlist::create_video))
                .route("/playlist/dangerous-actions/{site_path:.*}", get().to(playlist::dangerous_actions))
                .route("/playlist/delete/{site_path:.*}", post().to(playlist::delete))
                .route("/playlist/delete-banner/{site_path:.*}", post().to(playlist::delete_banner))
                .route("/playlist/delete-link/{site_path:.*}", post().to(playlist::delete_link))
                .service(
                    resource("/playlist/edit/{site_path:.*}")
                        .route(get().to(playlist::edit))
                        .route(post().to(playlist::update))
                )
                .service(
                    resource("/playlist/manifest/{site_path:.*}")
                        .route(get().to(playlist::edit_manifest))
                        .route(post().to(playlist::update_manifest))
                )
                .route("/playlist/move/{site_path:.*}", post().to(playlist::r#move))
                .route("/playlist/update-banner/{site_path:.*}", post().to(playlist::update_banner))
                .route("/playlist/upload-banner/{site_path:.*}", post().to(playlist::upload_banner))
                // .route("/playlist/{id}/add", post().to(playlist::add_video))
                // .route("/playlists", get().to(index::playlists))
                .route("/poster/{site_path:.*}", get().to(video::poster))
                .service(
                    // Static file serving of built site
                    Files::new("/preview/", &data_moving.build_dir)
                        .redirect_to_slash_directory()
                        .index_file("index.html")
                )
                .route("/processing", get().to(processing))
                .route("/scripts.js", get().to(scripts))
                .service(
                    resource("/site/edit/")
                        .route(get().to(site::edit))
                        .route(post().to(site::update))
                )
                .service(
                    resource("/site/manifest/")
                        .route(get().to(site::edit_manifest))
                        .route(post().to(site::update_manifest))
                )
                .route("/splash_logo.svg", get().to(splash_logo))
                .route("/styles.css", get().to(styles))
                .route("/styles.css", get().to(styles))
                .route("/video/add-link/{site_path:.*}", post().to(video::add_link))
                .route("/video/dangerous-actions/{site_path:.*}", get().to(video::dangerous_actions))
                .route("/video/delete/{site_path:.*}", post().to(video::delete))
                .route("/video/delete-link/{site_path:.*}", post().to(video::delete_link))
                .route("/video/delete-poster/{site_path:.*}", post().to(video::delete_poster))
                .route("/video/delete-version/{site_path:.*}", post().to(video::delete_version))
                .route("/video/downsize/{site_path:.*}", post().to(video::downsize))
                .service(
                    resource("/video/edit/{site_path:.*}")
                        .route(get().to(video::edit))
                        .route(post().to(video::update))
                )
                .service(
                    resource("/video/manifest/{site_path:.*}")
                        .route(get().to(video::edit_manifest))
                        .route(post().to(video::update_manifest))
                )
                .route("/video/media/{site_path:.*}", get().to(video::media))
                .route("/video/move/{site_path:.*}", post().to(video::r#move))
                .route("/video/random-poster/{site_path:.*}", post().to(video::random_poster))
                .route("/video/subtitles/{site_path:.*}", get().to(video::subtitles))
                // .route("/videos", get().to(index::videos))
                .route("/video/update-poster/{site_path:.*}", post().to(video::update_poster))
                .route("/video/upload-poster/{site_path:.*}", post().to(video::upload_poster))
                .route("/video/upload-video/{site_path:.*}", post().to(video::upload_video))
        })
        .bind((ip, port))
    };

    let data = Data::new(context);

    let ip = ip_requested.unwrap_or(DEFAULT_EDITOR_IP);

    let (server, port_bound) = if let Some(port) = port_requested {
        match bind_server(data, ip, *port) {
            Ok(server) => (server, *port),
            Err(err) => {
                eprintln!("Could not bind editor server to {ip}:{port} ({err})");
                return
            }
        }
    } else {
        let mut port = DEFAULT_EDITOR_PORT;

        loop {
            match bind_server(data.clone(), ip, port) {
                Ok(server) => break (server, port),
                Err(err) => {
                    if port > DEFAULT_EDITOR_PORT + MAX_PORT_ATTEMPTS {
                        eprintln!("Could not bind preview server to {ip} on any port in the range {DEFAULT_EDITOR_PORT}-{port} (last error was {err}).\nUse --port to manually set a port and/or check that no other issue is at play");
                        return
                    }

                    port += 1;
                }
            }
        }
    };

    let url = format!("http://{ip}:{port_bound}");

    println!("Serving the editor at {url} (open this address in your browser if it doesn't by itself)");
    println!("Press Ctrl+C to shut down the editor (if you have pending encoding jobs they will be discarded, otherwise this is safe to do at any point)");

    let open_browser = || async {
        if webbrowser::open(&url).is_err() {
            eprintln!("Could not automatically open browser with the editor, please do so manually")
        }
    };

    tokio::join!(server.run(), open_browser()).0.unwrap();
}

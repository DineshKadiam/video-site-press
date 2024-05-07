use std::sync::Arc;

use actix_files::Files;
use actix_web::{App, HttpServer};
use actix_web::web::{Data, get, post, resource};

use crate::Context;
use crate::editor::endpoints::{
    build,
    change_editor_config,
    collection,
    convert,
    deploy,
    favicon,
    font,
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

const EDITOR_IP: &str = "127.0.0.1";
const DEFAULT_EDITOR_PORT: u16 = 8080;

/// When DEFAULT_EDITOR_PORT is unavailable, we try DEFAULT_EDITOR_PORT + 1,
/// then DEFAULT_EDITOR_PORT + 2, etc., but after MAX_PORT_ATTEMPTS
/// iterations we stop, as probably something is else is wrong then, rather
/// than all ports taken.
const MAX_PORT_ATTEMPTS: u16 = 10;

#[actix_web::main]
pub async fn start(context: Arc<Context>, port_requested: &Option<u16>) {
    let bind_server = |data_moving: Data<Arc<Context>>, port: u16| {
        HttpServer::new(move || {
            App::new()
                .app_data(data_moving.clone())
                .route("/", get().to(index::index))
                .route("/build", post().to(build))
                .route("/convert-to-collection/{path:.*}", post().to(convert::to_collection))
                .route("/convert-to-playlist/{path:.*}", post().to(convert::to_playlist))
                .route("/convert-to-video/{path:.*}", post().to(convert::to_video))
                .route("/change-editor-config", post().to(change_editor_config))
                .route("/create-subcollection/{path:.*}", post().to(collection::create_subcollection))
                .route("/create-playlist/{path:.*}", post().to(collection::create_playlist))
                .service(
                    resource("/collection/{path:.*}")
                        .route(get().to(collection::edit))
                        .route(post().to(collection::update)),
                )
                .route("/delete-banner/{path:.*}", post().to(playlist::delete_banner))
                .route("/delete-collection/{path:.*}", post().to(collection::delete))
                .route("/delete-playlist/{path:.*}", post().to(playlist::delete))
                .route("/delete-poster/{path:.*}", post().to(video::delete_poster))
                .route("/delete-version/{path:.*}", post().to(video::delete_version))
                .route("/delete-video/{path:.*}", post().to(video::delete))
                .route("/deploy", post().to(deploy))
                .route("/downsize/{path:.*}", post().to(video::downsize))
                .route("/favicon.png", get().to(favicon))
                // .route("/formats", post().to(settings::formats))
                .route("/jobs", get().to(jobs))
                .route("/move-collection/{path:.*}", post().to(collection::r#move))
                .route("/move-playlist/{path:.*}", post().to(playlist::r#move))
                .route("/move-video/{path:.*}", post().to(video::r#move))
                // .route("/playlist", post().to(playlist::create))
                .service(
                    resource("/playlist/{path:.*}")
                        .route(get().to(playlist::edit))
                        .route(post().to(playlist::update)),
                )
                // .route("/playlist/{id}/add", post().to(playlist::add_video))
                // .route("/playlists", get().to(index::playlists))
                .route("/poster/{path:.*}", get().to(video::poster))
                .service(
                    // Static file serving of built site
                    Files::new("/preview/", &data_moving.build_dir)
                        .index_file("index.html")
                        .show_files_listing()
                )
                .route("/processing", get().to(processing))
                .route("/random-poster/{path:.*}", post().to(video::random_poster))
                .route("/roboto-v30-latin_latin-ext-regular.woff2", get().to(font))
                .route("/site", post().to(site::update))
                .route("/scripts.js", get().to(scripts))
                .route("/splash_logo.svg", get().to(splash_logo))
                .route("/styles.css", get().to(styles))
                .service(
                    resource("/video/{path:.*}")
                        .route(get().to(video::edit))
                        .route(post().to(video::update)),
                )
                // .route("/videos", get().to(index::videos))
                .route("/update-banner/{path:.*}", post().to(playlist::update_banner))
                .route("/upload-banner/{path:.*}", post().to(playlist::upload_banner))
                .route("/update-poster/{path:.*}", post().to(video::update_poster))
                .route("/upload-poster/{path:.*}", post().to(video::upload_poster))
                .route("/upload-video/{path:.*}", post().to(video::upload_video))
        })
        .bind((EDITOR_IP, port))
    };

    let data = Data::new(context);

    let (server, port_bound) = if let Some(port) = port_requested {
        match bind_server(data, *port) {
            Ok(server) => (server, *port),
            Err(err) => {
                eprintln!("Could not bind editor server to {EDITOR_IP}:{port} ({err})");
                return
            }
        }
    } else {
        let mut port = DEFAULT_EDITOR_PORT;

        loop {
            match bind_server(data.clone(), port) {
                Ok(server) => break (server, port),
                Err(err) => {
                    if port > DEFAULT_EDITOR_PORT + MAX_PORT_ATTEMPTS {
                        eprintln!("Could not bind preview server to {EDITOR_IP} on any port in the range {DEFAULT_EDITOR_PORT}-{port} (last error was {err}).\nUse --port to manually set a port and/or check that no other issue is at play");
                        return
                    }

                    port += 1;
                }
            }
        }
    };

    let url = format!("http://localhost:{port_bound}");

    println!("Serving the editor at {url} (open this address in your browser if it doesn't by itself)");

    let open_browser = || async {
        if webbrowser::open(&url).is_err() {
            eprintln!("Could not automatically open browser with the editor, please do so manually")
        }
    };

    tokio::join!(server.run(), open_browser()).0.unwrap();
}
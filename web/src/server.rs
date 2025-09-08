// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;

use actix_files::Files;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer};
use actix_web::cookie::Key;
use actix_web::web::{Data, get, post, resource};
use actix_web::rt;

use hyper8_core::Context as CoreContext;
use hyper8_core::util::ensure_dir;

use crate::Context;
use crate::endpoints::{
    build,
    change_language_and_theme,
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

/// Returns either the url of the started server or a message why starting the
/// server failed
pub fn start(
    core_context: Arc<CoreContext>,
    ip_requested: Option<IpAddr>,
    port_requested: Option<u16>
) -> Result<String, String> {
    // We ensure (and if present beforehand, wipe) the build directory on
    // build anyway, but as actix-files does not successfully register
    // the /preview/ static file hosting route if the directory does not
    // exist on startup, we create it here already.
    ensure_dir(&core_context.build_dir);

    let context = Arc::new(Context::new(core_context));
    let data = Data::new(context);

    let ip = ip_requested.unwrap_or(DEFAULT_EDITOR_IP);

    let (sender, receiver) = channel();

    thread::spawn(move || {
        let bind_server = |data_moving: Data<Arc<Context>>, ip: IpAddr, port: u16| {
            HttpServer::new(move || {
                App::new()
                    .wrap(
                        SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                            .cookie_secure(false)
                            .build()
                    )
                    .app_data(data_moving.clone())
                    .route("/", get().to(index::index))
                    .route("/build/{site_path:.*}", post().to(build))
                    .route("/convert-to-collection/{site_path:.*}", post().to(convert::to_collection))
                    .route("/convert-to-playlist/{site_path:.*}", post().to(convert::to_playlist))
                    .route("/convert-to-video/{site_path:.*}", post().to(convert::to_video))
                    .route("/change-language-and-theme", post().to(change_language_and_theme))
                    .route("/change-tree-view-order", post().to(change_tree_view_order))
                    .route("/collection/add-link/{site_path:.*}", post().to(collection::add_link))
                    .route("/collection/banner/{site_path:.*}", get().to(collection::banner))
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
                    .route("/collection/links/{site_path:.*}", get().to(collection::links))
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
                    .route("/editor.css", get().to(styles))
                    .route("/editor.js", get().to(scripts))
                    .route("/favicon.png", get().to(favicon))
                    .route("/jobs", get().to(jobs))
                    .route("/playlist/add-link/{site_path:.*}", post().to(playlist::add_link))
                    .route("/playlist/banner/{site_path:.*}", get().to(playlist::banner))
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
                    .route("/playlist/links/{site_path:.*}", get().to(playlist::links))
                    .service(
                        resource("/playlist/manifest/{site_path:.*}")
                            .route(get().to(playlist::edit_manifest))
                            .route(post().to(playlist::update_manifest))
                    )
                    .route("/playlist/move/{site_path:.*}", post().to(playlist::r#move))
                    .route("/playlist/update-banner/{site_path:.*}", post().to(playlist::update_banner))
                    .route("/playlist/upload-banner/{site_path:.*}", post().to(playlist::upload_banner))
                    // TODO: This is strictly speaking not a site_path, but right now it's convenient
                    //       to use that abstraction, because it handles path separator normalization
                    //       across platforms. (also applies to video_stream)
                    .route("/poster/{site_path:.*}", get().to(video::poster))
                    .service(
                        // Static file serving of built site
                        Files::new("/preview/", &data_moving.build_dir)
                            .redirect_to_slash_directory()
                            .index_file("index.html")
                    )
                    .route("/processing", get().to(processing))
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
                    .route("/video/add-link/{site_path:.*}", post().to(video::add_link))
                    .route("/video/dangerous-actions/{site_path:.*}", get().to(video::dangerous_actions))
                    .route("/video/delete/{site_path:.*}", post().to(video::delete))
                    .route("/video/delete-audio-file/{site_path:.*}", post().to(video::delete_audio_file))
                    .route("/video/delete-video-file/{site_path:.*}", post().to(video::delete_video_file))
                    .route("/video/delete-link/{site_path:.*}", post().to(video::delete_link))
                    .route("/video/delete-poster/{site_path:.*}", post().to(video::delete_poster))
                    .route("/video/delete-subtitles/{site_path:.*}", post().to(video::delete_subtitles))
                    .route("/video/encode-audio/{site_path:.*}", post().to(video::encode_audio))
                    .route("/video/encode-video/{site_path:.*}", post().to(video::encode_video))
                    .service(
                        resource("/video/edit/{site_path:.*}")
                            .route(get().to(video::edit))
                            .route(post().to(video::update))
                    )
                    .service(
                        resource("/video/edit-subtitles/{file_name}/{site_path:.*}")
                            .route(get().to(video::edit_subtitles))
                            .route(post().to(video::update_subtitles))
                    )
                    .route("/video/links/{site_path:.*}", get().to(video::links))
                    .service(
                        resource("/video/manifest/{site_path:.*}")
                            .route(get().to(video::edit_manifest))
                            .route(post().to(video::update_manifest))
                    )
                    .route("/video/media/{site_path:.*}", get().to(video::media))
                    .route("/video/move/{site_path:.*}", post().to(video::r#move))
                    .route("/video/pick-poster/{site_path:.*}", post().to(video::pick_poster))
                    .route("/video/poster/{site_path:.*}", get().to(video::edit_poster))
                    .route("/video/stream/{site_path:.*}", get().to(video::stream))
                    .route("/video/subtitles/{site_path:.*}", get().to(video::subtitles))
                    .route("/video/update-poster/{site_path:.*}", post().to(video::update_poster))
                    .route("/video/upload-poster/{site_path:.*}", post().to(video::upload_poster))
                    .route("/video/upload-subtitles/{site_path:.*}", post().to(video::upload_subtitles))
                    .route("/video/upload-audio-video/{site_path:.*}", post().to(video::upload_audio_video))
            })
            .bind((ip, port))
        };

        let (server, port_bound) = if let Some(port) = port_requested {
            match bind_server(data, ip, port) {
                Ok(server) => (server, port),
                Err(err) => {
                    let message = format!("Could not bind editor server to {ip}:{port} ({err})");
                    sender.send(Err(message)).unwrap();
                    return;
                }
            }
        } else {
            let mut port = DEFAULT_EDITOR_PORT;

            loop {
                match bind_server(data.clone(), ip, port) {
                    Ok(server) => break (server, port),
                    Err(err) => {
                        if port > DEFAULT_EDITOR_PORT + MAX_PORT_ATTEMPTS {
                            let message = format!("Could not bind preview server to {ip} on any port in the range {DEFAULT_EDITOR_PORT}-{port} (last error was {err}).\nUse --port to manually set a port and/or check that no other issue is at play");
                            sender.send(Err(message)).unwrap();
                            return;
                        }

                        port += 1;
                    }
                }
            }
        };

        let url = format!("http://{ip}:{port_bound}");

        let send_url = || async { sender.send(Ok(url)).unwrap(); };
        rt::System::new().block_on(async move {
            // Using `biased` here is an attempt to make (more) sure that running the server
            // takes the first turn and send_url is only invoked once the server first hands
            // over the thread (because it's done initializing).
            // (see https://docs.rs/tokio/latest/tokio/macro.join.html#fairness)
            tokio::join!(
                biased;
                server.run(),
                send_url()
            ).0.unwrap()
        });
    });

    // The thread in which the server runs returns a single value over the
    // channel, either the url of the server or the error why it could not be
    // started.
    receiver.recv().unwrap()
}

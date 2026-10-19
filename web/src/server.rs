// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use actix_files::Files;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer};
use actix_web::cookie::Key;
use actix_web::rt;
use actix_web::web::{Data, FormConfig};
use actix_web::web::{get, post, resource};
use indoc::formatdoc;

use hyper8_core::Context as CoreContext;
use hyper8_core::DEFAULT_IP;
use hyper8_core::{default_ports_display, default_ports_iter};
use hyper8_core::util::ensure_dir;

use crate::{Context, EditorState};
use crate::endpoints::{
    build_with_design_overlay,
    build_without_design_overlay,
    collection,
    convert,
    editor,
    favicon,
    index,
    jobs,
    playlist,
    processing,
    scripts,
    styles,
    site,
    sync,
    sync_log,
    video
};

pub enum ServerMode {
    MultipleRemoteUsers {
        inherited_editor_state: EditorState
    },
    SingleLocalUser {
        synced_editor_state: Arc<Mutex<EditorState>>
    }
}

/// Returns either the url of the started server or a message why starting the
/// server failed
pub fn start(
    core_context: Arc<CoreContext>,
    ip_requested: Option<IpAddr>,
    port_requested: Option<u16>,
    server_mode: ServerMode
) -> Result<String, String> {
    // We ensure (and if present beforehand, wipe) the build directory on
    // build anyway, but as actix-files does not successfully register
    // the /preview/ static file hosting route if the directory does not
    // exist on startup, we create it here already.
    ensure_dir(&core_context.build_dir);

    let context = Context::new(core_context, server_mode);
    let context = Arc::new(context);
    let data = Data::new(context);

    let ip = ip_requested.unwrap_or(DEFAULT_IP);

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
                    .route("/build/{site_path:.*}", get().to(build_without_design_overlay))
                    .route("/build-design", get().to(build_with_design_overlay))
                    .route("/convert-to-collection/{site_path:.*}", post().to(convert::to_collection))
                    .route("/convert-to-playlist/{site_path:.*}", post().to(convert::to_playlist))
                    .route("/convert-to-video/{site_path:.*}", post().to(convert::to_video))
                    .route("/change-language", post().to(editor::change_language))
                    .route("/change-theme", post().to(editor::change_theme))
                    .route("/change-tree-view-order", post().to(editor::change_tree_view_order))
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
                    .route("/processing", get().to(processing::processing_general))
                    .route("/processing/media/{site_path:.*}", get().to(processing::processing_media))
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
                    .route("/sync/ftp", post().to(sync::sync_ftp))
                    .route("/sync/ftp/check", post().to(sync::check_ftp))
                    .route("/sync/ftp/config", get().to(sync::config_ftp))
                    .route("/sync/ftp/simulate", post().to(sync::simulate_ftp))
                    .route("/sync/ssh", post().to(sync::sync_ssh))
                    .route("/sync/ssh/check", post().to(sync::check_ssh))
                    .route("/sync/ssh/config", get().to(sync::config_ssh))
                    .route("/sync/ssh/simulate", post().to(sync::simulate_ssh))
                    .route("/sync_log", get().to(sync_log))
                    .route("/video/add-link/{site_path:.*}", post().to(video::add_link))
                    .route("/video/chapters/{site_path:.*}", get().to(video::chapters))
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
                            // Raises the limit for URL encoded form payloads from 16kB to 512kB
                            // to allow subtitle files with lots of content to be edited/submitted.
                            .app_data(FormConfig::default().limit(512 * 1024))
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
                    .route("/video/update-audio-file-usage/{site_path:.*}", post().to(video::update_audio_file_usage))
                    .route("/video/update-chapters/{site_path:.*}", post().to(video::update_chapters))
                    .route("/video/update-poster/{site_path:.*}", post().to(video::update_poster))
                    .route("/video/update-video-file-usage/{site_path:.*}", post().to(video::update_video_file_usage))
                    .route("/video/upload-audio-video/{site_path:.*}", post().to(video::upload_audio_video))
                    .route("/video/upload-poster/{site_path:.*}", post().to(video::upload_poster))
                    .route("/video/upload-subtitles/{site_path:.*}", post().to(video::upload_subtitles))
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
            let mut default_ports = default_ports_iter().peekable();

            loop {
                let port = default_ports.next().unwrap();

                match bind_server(data.clone(), ip, port) {
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

                            sender.send(Err(message)).unwrap();

                            return;
                        }
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

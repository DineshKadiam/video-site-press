use std::sync::Arc;

use actix_files::Files;
use actix_web::{App, HttpServer};
use actix_web::web::{Data, get, post, resource};

use crate::Context;
use crate::editor::endpoints::{
    build,
    collection,
    convert,
    deploy,
    favicon,
    font,
    index,
    playlist,
    processing,
    scripts,
    styles,
    site,
    video
};

#[actix_web::main]
pub async fn start(context: Arc<Context>) {
    let data = Data::new(context);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", get().to(index::index))
            .route("/build", post().to(build))
            .route("/convert/collection", post().to(convert::collection))
            .route("/convert/playlist", post().to(convert::playlist))
            .route("/convert/video", post().to(convert::video))
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
            .route("/delete-playlist-video/{path:.*}", post().to(playlist::remove_video))
            .route("/delete-video/{path:.*}", post().to(video::delete))
            .route("/deploy", post().to(deploy))
            .route("/downsize/{path:.*}", post().to(video::downsize))
            .route("/favicon.png", get().to(favicon))
            // .route("/formats", post().to(settings::formats))
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
                Files::new("/preview/", &data.build_dir)
                    .index_file("index.html")
                    .show_files_listing()
            )
            .route("/processing", get().to(processing))
            .route("/roboto-v30-latin_latin-ext-regular.woff2", get().to(font))
            .route("/site", post().to(site::update))
            .route("/scripts.js", get().to(scripts))
            .route("/styles.css", get().to(styles))
            .service(
                resource("/video/{path:.*}")
                    .route(get().to(video::edit))
                    .route(post().to(video::update)),
            )
            // .route("/videos", get().to(index::videos))
            .route("/update-banner/{path:.*}", post().to(playlist::update_banner))
            .route("/upload-banner/{path:.*}", post().to(playlist::upload_banner))
            .route("/upload-poster/{path:.*}", post().to(video::upload_poster))
            .route("/upload-video/{path:.*}", post().to(video::upload_video))
    })
    .bind(("127.0.0.1", 8081))
    .unwrap()
    .run()
    .await.unwrap()
}
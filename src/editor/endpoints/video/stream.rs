// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath};

use crate::{Context, SitePath};

// TODO: We should make this a little more secure (catch the worst path
// injection attack stuff) because someone is going to leave this open
// to the net at some point.
pub async fn stream(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let filepath = context.site_dir.join(site_path.filesystem_relative());
    let filepath_moving = filepath.clone();
    let content = web::block(move || fs::read(filepath_moving))
        .await
        .unwrap()
        .unwrap();

    match filepath.extension() {
        Some(extension) => {
            // TODO: This somewhat duplicates logic we have in VideoFile::mime_type,
            //       possibly unify at some point.
            let content_type = if extension.eq_ignore_ascii_case("m4v") || extension.eq_ignore_ascii_case("mp4") {
                "video/mp4"
            } else if extension.eq_ignore_ascii_case("mkv") {
                "video/x-matroska"
            } else if extension.eq_ignore_ascii_case("mov") {
                "video/quicktime"
            } else if extension.eq_ignore_ascii_case("webm") {
                "video/webm"
            } else {
                let body = format!("Video stream file extension {} can not be requested.", extension.to_string_lossy());
                return HttpResponse::NotImplemented().body(body);
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .body(content)
        }
        None => {
            HttpResponse::NotImplemented()
                .body("Files without extension can not be requested as a video stream.")
        }
    }
}

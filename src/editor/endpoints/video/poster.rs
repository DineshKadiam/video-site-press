// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath};

use crate::{Context, SitePath};

// TODO: We currently go through the video endpoint for poster for all posters
//       (also playlist e.g.), this is unclean, make generic or change otherwise
// TODO: We should make this a little more secure (catch the worst path
// injection attack stuff) because someone is going to leave this open
// to the net at some point.
pub async fn poster(
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
            let content_type = if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpg") {
                "image/jpeg"
            } else if extension.eq_ignore_ascii_case("png") {
                "image/png"
            } else if extension.eq_ignore_ascii_case("webp") {
                "image/webp"
            } else if extension.eq_ignore_ascii_case("webp") {
                "image/gif"
            } else {
                let body = format!("Poster file extension {} can not be requested.", extension.to_string_lossy());
                return HttpResponse::NotImplemented().body(body);
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .body(content)
        }
        None => {
            HttpResponse::NotImplemented()
                .body("Files without extension can not be requested as a poster.")
        }
    }
}

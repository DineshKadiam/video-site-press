// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;

/// Wraps the passed String into an HttpResponse with content type html
pub fn html_response(html: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

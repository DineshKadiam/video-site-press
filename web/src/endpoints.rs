// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_web::HttpResponse;

mod build;
mod deployment;
mod jobs;
mod scripts;
mod styles;

pub mod collection;
pub mod convert;
pub mod deploy;
pub mod editor;
pub mod index;
pub mod playlist;
pub mod processing;
pub mod site;
pub mod video;

pub use build::build;
pub use deployment::deployment;
pub use jobs::jobs;
pub use scripts::scripts;
pub use styles::styles;

pub async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .body(include_bytes!("assets/favicon.png").as_slice())
}

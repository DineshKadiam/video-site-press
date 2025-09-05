// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::web::{Form, Redirect};
use serde_derive::Deserialize;

pub mod collection;
pub mod convert;
mod build;
pub mod deploy;
mod deployment;
pub mod index;
mod jobs;
pub mod playlist;
mod processing;
pub mod site;
mod scripts;
mod styles;
pub mod video;

pub use build::build;
pub use deployment::deployment;
pub use jobs::jobs;
pub use processing::processing;
pub use scripts::scripts;
pub use styles::styles;

use crate::Session;

#[derive(Deserialize)]
pub struct LanguageAndThemeForm {
    language_code: String,
    redirect: String,
    theme_key: String
}

#[derive(Deserialize)]
pub struct TreeViewOrderForm {
    redirect: String
}

pub async fn change_language_and_theme(
    form: Form<LanguageAndThemeForm>,
    session: ActixSession
) -> Redirect {
    let _ = session.insert("language_code", &form.language_code);
    let _ = session.insert("theme_key", &form.theme_key);

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn change_tree_view_order(
    form: Form<TreeViewOrderForm>,
    session: ActixSession
) -> Redirect {
    let tree_view_order = Session::tree_view_order(&session);

    let _ = session.insert("tree_view_order_key", tree_view_order.cycle().key());

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .body(include_bytes!("assets/favicon.png").as_slice())
}

pub async fn splash_logo() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(include_bytes!("assets/splash_logo.svg").as_slice())
}

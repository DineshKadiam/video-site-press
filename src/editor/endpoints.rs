use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Form, Redirect};
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

use crate::{Context, EditorConfig, Language, Theme};

#[derive(Deserialize)]
pub struct EditorConfigForm {
    language_code: String,
    redirect: String,
    theme_key: String
}

pub async fn change_editor_config(
    context: Data<Arc<Context>>,
    form: Form<EditorConfigForm>,
) -> Redirect {
    *context.editor_language.lock().unwrap() = Language::from_code(&form.language_code);
    *context.editor_theme.lock().unwrap() = Theme::from_key(&form.theme_key).unwrap_or_else(|_| Theme::adaptive());

    let editor_config = EditorConfig::new(&form.language_code, &form.theme_key);

    editor_config.persist();

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

use actix_web::HttpResponse;

pub mod collection;
pub mod convert;
mod build;
mod deploy;
pub mod index;
pub mod playlist;
mod processing;
pub mod site;
pub mod video;

pub use build::build;
pub use deploy::deploy;
pub use processing::processing;

pub async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .body(include_bytes!("assets/favicon.png").as_slice())
}

pub async fn font() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("font/woff2")
        .body(include_bytes!("assets/roboto-v30-latin_latin-ext-regular.woff2").as_slice())
}

pub async fn scripts() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!("assets/scripts.js"))
}

pub async fn styles() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(include_str!("assets/styles.css"))
}
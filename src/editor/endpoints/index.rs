use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, SiteContent};
use crate::editor::widgets::layout;

pub async fn index(context: Data<Arc<Context>>) -> HttpResponse {
    let site = context.get_site();

    let mut body = match &site.content {
        SiteContent::Collection(_) => {
            formatdoc!(r#"
                This site is operating as a collection
            "#)
        }
        SiteContent::Empty => {
            formatdoc!(r#"
                <h2>Empty site</h2>

                Initialize your site by placing content in the site directory,
                or picking one of the options below:<br><br>

                <form action="/convert/collection" method="post"><button>Initialize site as collection</button></form> As a collection the site holds videos, playlists and recursive subcollections.<br><br>

                <form action="/convert/playlist" method="post"><button>Initialize site as playlist</button></form> A site featuring a number of videos, presented as an ordered viewing list<br><br>

                <form action="/convert/video" method="post"><button>Initialize site as video</button></form> A site presenting just a single video<br><br>

            "#)
        }
        SiteContent::Invalid => {
            "Site is invalid - site folder content needs to be manually fixed.".to_string()
        }
        SiteContent::Playlist(_) => {
            formatdoc!(r#"
                This site is operating as a playlist
            "#)
        }
        SiteContent::Video(_) => {
            formatdoc!(r#"
                This site is operating as a video
            "#)
        }
    };

    let job_info = context.job_queue
        .lock()
        .unwrap()
        .iter()
        .map(|job| {
            format!(
                "{video} {status}",
                status = if job.assigned { "assigned" } else { "idle" },
                video = job.video.label()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    body.push_str(&job_info);

    if let Ok(processing_errors) = context.processing_errors.lock() {
        if !processing_errors.is_empty() {
            let errors = processing_errors
                .iter()
                .map(|err| format!("- {err}<br>"))
                .collect::<Vec<String>>()
                .join("\n");

            let wrapped = formatdoc!(r#"
                <div>
                    Processing errors:<br>
                    {errors}
                </div>
            "#);

            body.push_str(&wrapped);
        }
    }

    let html = layout(&context, "", &site, &body, site.title());

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, JobKind, SiteContent};
use crate::editor::widgets::layout;

pub async fn index(context: Data<Arc<Context>>) -> HttpResponse {
    let site = context.get_site();

    let mut body = match &site.content {
        SiteContent::Collection(_) => {
            formatdoc!(r#"
                <div class="form_group">
                    This site is operating as a collection
                </div>
            "#)
        }
        SiteContent::Empty => {
            formatdoc!(r#"
                <div class="form_group">
                    <h2>Empty site</h2>

                    Initialize your site by placing content in the site directory,
                    or picking one of the options below:<br><br>

                    <form action="/convert-to-collection/" method="post"><button>Initialize site as collection</button></form> As a collection the site holds videos, playlists and recursive subcollections.<br><br>

                    <form action="/convert-to-playlist/" method="post"><button>Initialize site as playlist</button></form> A site featuring a number of videos, presented as an ordered viewing list<br><br>

                    <form action="/convert-to-video/" method="post"><button>Initialize site as video</button></form> A site presenting just a single video
                </div>
            "#)
        }
        SiteContent::Invalid => {
            formatdoc!(r#"
                <div class="form_group">
                    Site is invalid - site folder content needs to be manually fixed.
                </div>
            "#)
        }
        SiteContent::Playlist(_) => {
            formatdoc!(r#"
                <div class="form_group">
                    This site is operating as a playlist
                </div>
            "#)
        }
        SiteContent::Video(_) => {
            formatdoc!(r#"
                <div class="form_group">
                    This site is operating as a video
                </div>
            "#)
        }
    };

    let job_info = context.job_queue
        .lock()
        .unwrap()
        .iter()
        .map(|job| {
            let label = match &job.kind {
                JobKind::PlaylistFastTasks(playlist) => playlist.label(),
                JobKind::VideoFastTasks { video, .. } => video.label(),
                JobKind::VideoFormat { video, .. } => video.label(),
            };
            let status = if job.assigned { "assigned" } else { "idle" };

            format!("{label} {status}")
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

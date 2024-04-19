use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, JobKind, SiteContent};
use crate::editor::widgets::layout;
use crate::util::human_size;

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
            let t_empty_site = &context.editor_translations.empty_site;
            let t_initialize_site_as_collection = &context.editor_translations.initialize_site_as_collection;
            let t_initialize_site_as_playlist = &context.editor_translations.initialize_site_as_playlist;
            let t_initialize_site_as_video = &context.editor_translations.initialize_site_as_video;
            formatdoc!(r#"
                <div class="form_group">
                    <h2>{t_empty_site}</h2>

                    Initialize your site by placing content in the site directory,
                    or picking one of the options below:<br><br>

                    <form action="/convert-to-collection/" method="post">
                        <button>{t_initialize_site_as_collection}</button> As a collection the site holds videos, playlists and recursive subcollections.
                    </form>

                    <br><br>

                    <form action="/convert-to-playlist/" method="post">
                        <button>{t_initialize_site_as_playlist}</button> A site featuring a number of videos, presented as an ordered viewing list
                    </form>

                    <br><br>

                    <form action="/convert-to-video/" method="post">
                        <button>{t_initialize_site_as_video}</button> A site presenting just a single video
                    </form>
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
        SiteContent::Playlist(playlist) => {
            // TODO: Group entries by collection/playlist/video (maybe multiple rings? or multiple charts?)
            // TODO: Wire up chart hover with tabular row hover and vice versa
            // TODO: Tweak values for pie chart

            let radius = 190;
            let diameter = radius * 2;

            let mut sizes = Vec::new();
            let mut total_size: u64 = 0;

            for video in &playlist.videos {
                for version in &video.versions {
                    let path = &video.path;
                    let link = format!("/video/{path}");
                    let size = version.file_meta.size;
                    let title = format!("{} ({})", version.file_name, video.label(&context));
                    sizes.push((size, link, title));
                    total_size += size;
                }
            }
            
            sizes.sort_unstable_by(|a, b| b.0.cmp(&a.0));

            let mut amount_consumed = 0.0;

            // amount_of_total and offset both between 0-1
            let slice = |amount_of_total: f32, link: &str, odd: bool, offset: f32, title: &str| -> String {
                let angle = amount_of_total * 360.0;
                let radians = angle.to_radians();
                let rotation = 180.0 + (offset * -360.0);

                let outer_arc_end_x = radians.sin() * radius as f32 + radius as f32;
                let outer_arc_end_y = radians.cos() * radius as f32 + radius as f32;
                let large_arc_flag = if angle < 180.0 { 0 } else { 1 };

                let fill = if odd { "var(--fg-3)" } else { "var(--bg-3)" };

                let inner_radius = radius as f32 * 0.667;
                let inner_arc_begin_x = radians.sin() * inner_radius as f32 + radius as f32;
                let inner_arc_begin_y = radians.cos() * inner_radius as f32 + radius as f32;
                let inner_arc_end_y = radius as f32 + inner_radius;

                formatdoc!(r#"
                        <path d="
                            M {radius},{diameter}
                            A {radius} {radius} 0 {large_arc_flag} 0 {outer_arc_end_x},{outer_arc_end_y}
                            L {inner_arc_begin_x},{inner_arc_begin_y}
                            A {inner_radius} {inner_radius} 0 {large_arc_flag} 1 {radius},{inner_arc_end_y}
                            Z
                            " 
                            fill="{fill}"
                            onclick="window.location = '{link}';"
                            transform="rotate({rotation} {radius} {radius})">
                            <title>{title}</title>
                        </path>
                "#)
            };

            let mut list = String::new();
            let mut odd = true;
            let mut slices = String::new();

            for (size, link, title) in &sizes {
                let amount_of_total = *size as f32 / total_size as f32;

                let list_text = format!("{} {title}<br>", human_size(*size));

                list.push_str(&list_text);

                slices.push_str(&slice(amount_of_total, link, odd, amount_consumed, title));

                amount_consumed += amount_of_total;
                odd = !odd;
            }

            let total_size_human = human_size(total_size);

            formatdoc!(r#"
                <div class="form_group">
                    <style>
                        path {{ cursor: pointer; }}
                        path:hover {{ fill: var(--fg-1); }}
                    </style>
                    <svg width="{diameter}" height="{diameter}" version="1.1" viewBox="0 0 {diameter} {diameter}" xmlns="http://www.w3.org/2000/svg">
                        <g transform="scale(-1,1)" transform-origin="{radius} {radius}">
                            {slices}
                        </g>
                        <text stroke="currentColor" text-anchor="middle" x="{radius}" y="{radius}">
                            {total_size_human}
                        </text>
                    </svg>

                    <br><br>

                    {list}
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
                JobKind::PlaylistFastTasks(playlist) => playlist.label(&context),
                JobKind::VideoFastTasks { video, .. } => video.label(&context),
                JobKind::VideoFormat { video, .. } => video.label(&context),
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

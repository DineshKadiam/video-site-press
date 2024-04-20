use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Collection, Context, JobKind, SiteContent, Video};
use crate::editor::widgets::layout;
use crate::util::human_size;

pub async fn index(context: Data<Arc<Context>>) -> HttpResponse {
    let site = context.get_site();

    let mut body = match &site.content {
        SiteContent::Collection(collection) => {
            let mut videos = Vec::new();

            fn add_collection(collection: &Collection, videos: &mut Vec<Arc<Video>>) {
                for playlist in &collection.playlists {
                    for video in &playlist.videos {
                        videos.push(Arc::clone(video));
                    }
                }

                for subcollection in &collection.subcollections {
                    add_collection(subcollection, videos);
                }

                for video in &collection.videos {
                    videos.push(Arc::clone(video));
                }
            }

            add_collection(collection, &mut videos);


            let stats = storage_stats_html(&context, &videos);
            stats
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
            let stats = storage_stats_html(&context, &playlist.videos);
            stats
        }
        SiteContent::Video(video) => {
            let videos = vec![Arc::clone(video)];
            let stats = storage_stats_html(&context, &videos);
            stats
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

fn storage_stats_html(context: &Context, videos: &Vec<Arc<Video>>) -> String {
    // TODO: Group entries by collection/playlist/video (maybe multiple rings? or multiple charts?)
    // TODO: Wire up chart hover with tabular row hover and vice versa
    // TODO: Tweak values/readability for pie chart further (e.g. colors too as additional for those who see it)

    let radius = 190;
    let diameter = radius * 2;

    let mut sizes = Vec::new();
    let mut total_size: u64 = 0;

    for video in videos {
        for version in &video.versions {
            let path = &video.path;
            let link = format!("/video/{path}");
            let size = version.file_meta.size;
            let title = format!("{}: {}", video.label(&context), version.file_name);
            sizes.push((size, link, title));
            total_size += size;
        }
    }

    sizes.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    let mut amount_consumed = 0.0;

    // amount_of_total and offset both between 0-1
    let slice = |amount_of_total: f32, link: &str, lightness: f32, rotation_offset: f32, title: &str| -> String {
        let angle = amount_of_total * 360.0;
        let radians = angle.to_radians();
        let rotation = 180.0 + (rotation_offset * -360.0);

        let outer_arc_end_x = radians.sin() * radius as f32 + radius as f32;
        let outer_arc_end_y = radians.cos() * radius as f32 + radius as f32;
        let large_arc_flag = if angle < 180.0 { 0 } else { 1 };

        let fill = format!("hsl(0, 0%, {lightness}%)");

        let inner_radius = radius as f32 * 0.667;
        let inner_arc_begin_x = radians.sin() * inner_radius as f32 + radius as f32;
        let inner_arc_begin_y = radians.cos() * inner_radius as f32 + radius as f32;
        let inner_arc_end_y = radius as f32 + inner_radius;

        let marker_outer_radius = inner_radius - 5.0;
        let marker_outer_x = radians.sin() * marker_outer_radius as f32 + radius as f32;
        let marker_outer_y = radians.cos() * marker_outer_radius as f32 + radius as f32;

        let marker_inner_radius = marker_outer_radius - radius as f32 * 0.05;
        let marker_inner_x = radians.sin() * marker_inner_radius as f32 + radius as f32;
        let marker_inner_y = radians.cos() * marker_inner_radius as f32 + radius as f32;

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
                <path class="marker"
                    d="
                    M {marker_inner_x},{marker_inner_y}
                    L {marker_outer_x},{marker_outer_y}
                    "
                    transform="rotate({rotation} {radius} {radius})">
                </path>
        "#)
    };

    let mut list = String::new();
    let mut odd = true;
    let mut slices = String::new();

    for (index, (size, link, title)) in sizes.iter().enumerate() {
        let amount_of_total = if sizes.len() == 1 {
            0.999 // TODO: Our graph drawing code can not handle 360 degree segements yet so work around it for now with this hack
        } else {
            *size as f32 / total_size as f32
        };

        let offset_fill = index as f32 / (sizes.len() - 1) as f32;
        let lightness = 75.0 - offset_fill * 50.0;

        let size_humanized = human_size(*size);
        let list_text = formatdoc!(r#"
            <li>
                <div style="background-color: hsl(0, 0%, {lightness}%);"></div> {size_humanized} <a href="{link}">{title}</a>
            </li>
        "#);

        list.push_str(&list_text);

        slices.push_str(&slice(amount_of_total, link, lightness, amount_consumed, title));

        amount_consumed += amount_of_total;
        odd = !odd;
    }

    let total_size_human = human_size(total_size);

    formatdoc!(r#"
        <div class="form_group">
            <svg class="storage_graph" width="{diameter}" height="{diameter}" version="1.1" viewBox="0 0 {diameter} {diameter}" xmlns="http://www.w3.org/2000/svg">
                <g transform="scale(-1,1)" transform-origin="{radius} {radius}">
                    {slices}
                </g>
                <text text-anchor="middle" x="{radius}" y="{radius}">
                    {total_size_human}
                </text>
            </svg>

            <ul class="storage_legend">
                {list}
            </ul>
        </div>
    "#)
}
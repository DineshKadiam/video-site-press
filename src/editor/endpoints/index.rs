// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    SiteContent,
    SitePath,
    Translations,
    Video
};
use crate::editor::Location;
use crate::editor::routes;
use crate::editor::widgets::layout;
use crate::icons;
use crate::util::{html_escape_outside_attribute, human_size};

pub async fn index(context: Data<Arc<Context>>) -> HttpResponse {
    let language = context.get_editor_language();
    let site = context.get_site();

    let translations = &language.translations;

    let body = match &site.content {
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


            if videos.is_empty() {
                String::new()
            } else {
                storage_stats_html(translations, &videos)
            }
        }
        SiteContent::Empty => empty_site_html(translations),
        SiteContent::Invalid { error } => invalid_site_html(error, translations),
        SiteContent::Playlist(playlist) => {
            if playlist.videos.is_empty() {
                String::new()
            } else {
                storage_stats_html(translations, &playlist.videos)
            }
        }
        SiteContent::Video(video) => {
            let videos = vec![Arc::clone(video)];
            storage_stats_html(translations, &videos)
        }
    };

    let html = layout(
        &body,
        &context,
        &language,
        Location::AuxiliaryPage,
        &site,
        site.title()
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

fn empty_site_html(translations: &Translations) -> String {
    let collection_icon = icons::collection(translations.collection);
    let playlist_icon = icons::playlist(translations.playlist);
    let video_icon = icons::video(translations.video);

    let t_a_site_presenting_a_single_playlist_of_videos = translations.a_site_presenting_a_single_playlist_of_videos;
    let t_a_site_presenting_any_number_of_videos_and_playlists = translations.a_site_presenting_any_number_of_videos_and_playlists;
    let t_a_site_presenting_just_a_single_video = translations.a_site_presenting_just_a_single_video;
    let t_initialize_site_as_a_collection = translations.initialize_site_as_a_collection;
    let t_initialize_site_as_a_playlist = translations.initialize_site_as_a_playlist;
    let t_initialize_site_as_a_video = translations.initialize_site_as_a_video;
    let t_site_setup = translations.site_setup;
    let t_site_setup_change_note = translations.site_setup_change_note;
    let t_site_setup_text = translations.site_setup_text();

    let root_path = SitePath::root_path();
    let convert_to_collection_action = routes::convert_to_collection(&root_path);
    let convert_to_playlist_action = routes::convert_to_playlist(&root_path);
    let convert_to_video_action = routes::convert_to_video(&root_path);

    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_site_setup}</h2>
            <p>{t_site_setup_text}</p>
            <p style="background: var(--warning-bg); border-radius: .5rem; color: var(--warning-fg); padding: 1rem;">
                {t_site_setup_change_note}
            </p>
            <br>
            <form action="{convert_to_collection_action}" method="post">
                <button class="big">{collection_icon} {t_initialize_site_as_a_collection}</button>
                <p>{t_a_site_presenting_any_number_of_videos_and_playlists}</p>
            </form>
            <br>
            <form action="{convert_to_playlist_action}" method="post">
                <button class="big">{playlist_icon} {t_initialize_site_as_a_playlist}</button>
                <p>{t_a_site_presenting_a_single_playlist_of_videos}</p>
            </form>
            <br>
            <form action="{convert_to_video_action}" method="post">
                <button class="big">{video_icon} {t_initialize_site_as_a_video}</button>
                <p>{t_a_site_presenting_just_a_single_video}</p>
            </form>
        </div>
    "#)
}

fn invalid_site_html(error: &str, translations: &Translations) -> String {
    let failure_icon = icons::failure(translations.error);
    let t_invalid_site_text = translations.invalid_site_text();

    formatdoc!(r#"
        <p>{t_invalid_site_text}</p>
        <p class="error">{failure_icon} {error}</p>
    "#)
}

fn storage_stats_html(
    translations: &Translations,
    videos: &Vec<Arc<Video>>
) -> String {
    // TODO: Group entries by collection/playlist/video (maybe multiple rings? or multiple charts?)
    // TODO: Wire up chart hover with tabular row hover and vice versa
    // TODO: Tweak values/readability for pie chart further (e.g. colors too as additional for those who see it)

    let radius = 190.0;
    let diameter = radius * 2.0;

    let mut sizes = Vec::new();
    let mut total_size: u64 = 0;

    for video in videos {
        for version in &video.versions {
            let link = routes::video_edit(video);
            let size = version.file_meta.size;
            let title = format!("{}: {}", video.title_or_slug_or_generic_label(translations), version.file_name);
            let e_title = html_escape_outside_attribute(&title);
            sizes.push((size, link, e_title));
            total_size += size;
        }
    }

    sizes.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    // Draws a ring segment. In clock terms we start at 12 o'clock and we go clockwise.
    let slice = |arc_angle: f32, begin_angle: f32, link: &str, lightness: f32, title: &str| -> String {
        let fill = format!("hsl(0, 0%, {lightness}%)");
        let large_arc_flag = if arc_angle < 180.0 { 0 } else { 1 };

        let begin_angle_rad = begin_angle.to_radians();
        let begin_x = begin_angle_rad.sin();
        let begin_y = -begin_angle_rad.cos();

        let end_angle_rad = (begin_angle + arc_angle).to_radians();
        let end_x = end_angle_rad.sin();
        let end_y = -end_angle_rad.cos();

        let segment_outer_begin_x = radius + begin_x * radius;
        let segment_outer_begin_y = radius + begin_y * radius;

        let segment_outer_end_x = radius + end_x * radius;
        let segment_outer_end_y = radius + end_y * radius;

        let inner_radius = radius * 0.667;
        let segment_inner_begin_x = radius + begin_x * inner_radius;
        let segment_inner_begin_y = radius + begin_y * inner_radius;

        let segment_inner_end_x = radius + end_x * inner_radius;
        let segment_inner_end_y = radius + end_y * inner_radius;

        let marker_outer_radius = inner_radius - 5.0;
        let marker_outer_x = radius + end_x * marker_outer_radius;
        let marker_outer_y = radius + end_y * marker_outer_radius;

        let marker_inner_radius = marker_outer_radius - radius * 0.05;
        let marker_inner_x = radius + end_x * marker_inner_radius;
        let marker_inner_y = radius + end_y * marker_inner_radius;

        formatdoc!(r#"
                <path
                    d="M {segment_outer_begin_x},{segment_outer_begin_y}
                       A {radius} {radius} 0 {large_arc_flag} 1 {segment_outer_end_x},{segment_outer_end_y}
                       L {segment_inner_end_x},{segment_inner_end_y}
                       A {inner_radius} {inner_radius} 0 {large_arc_flag} 0 {segment_inner_begin_x},{segment_inner_begin_y}
                       Z"
                    fill="{fill}"
                    onclick="window.location = '{link}';">
                    <title>{title}</title>
                </path>
                <path class="marker"
                    d="M {marker_inner_x},{marker_inner_y} L {marker_outer_x},{marker_outer_y}">
                </path>
        "#)
    };

    let mut list = String::new();
    let mut slices = String::new();

    let mut begin_angle = 0.0;

    for (index, (size, link, title)) in sizes.iter().enumerate() {
        let arc_angle = if sizes.len() == 1 {
            359.0 // TODO: Our graph drawing code can not handle 360 degree segments yet so work around it for now with this hack
        } else {
            (*size as f32 / total_size as f32) * 360.0
        };

        let lightness_offset = if sizes.len() == 1 { 0.0 } else { index as f32 / (sizes.len() - 1) as f32 };
        let lightness = 75.0 - lightness_offset * 50.0;

        let size_humanized = human_size(*size);
        let list_text = formatdoc!(r#"
            <li>
                <div style="background: hsl(0, 0%, {lightness}%);"></div> {size_humanized} <a href="{link}">{title}</a>
            </li>
        "#);

        list.push_str(&list_text);

        slices.push_str(&slice(arc_angle, begin_angle, link, lightness, title));

        begin_angle += arc_angle;
    }

    let total_size_human = human_size(total_size);

    formatdoc!(r#"
        <svg class="storage_graph" width="{diameter}" height="{diameter}" version="1.1" viewBox="0 0 {diameter} {diameter}" xmlns="http://www.w3.org/2000/svg">
            {slices}
            <text text-anchor="middle" x="{radius}" y="{radius}">
                {total_size_human}
            </text>
        </svg>

        <ul class="storage_legend">
            {list}
        </ul>
    "#)
}
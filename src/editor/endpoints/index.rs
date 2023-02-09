use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use indoc::formatdoc;

use crate::{Context, SiteContent};
use crate::editor::widgets::{layout, upload_widget};
use crate::util::precise_duration;

// pub async fn playlists(context: Data<Arc<Context>>) -> HttpResponse {
//     let playlists = context.collection.all_playlists();
//     let playlists_rendered = playlists
//         .iter()
//         .map(|playlist| {
//             let count = &playlist.videos.len();
//             let id = &playlist.id;
//             let title = &playlist.title;

//             let unlisted = match playlist.unlisted {
//                 true => r#" <span class="badge">Unlisted</span>"#,
//                 false => ""
//             };

//             formatdoc!(r#"
//                 <div>
//                     <a href="/playlist/{id}">{title}</a>{unlisted}<br>
//                     {count} Videos
//                 </div>
//             "#)
//         })
//         .collect::<Vec<String>>()
//         .join("");

//     let controls_rendered = controls();

//     let body = formatdoc!(r#"
//         <div class="updates"></div>

//         {controls_rendered}

//         <h2>Playlists</h2>

//         <form action="/playlist" method="post" style="column-gap: .5rem; display: flex;">
//             <div>
//                 <input name="title" placeholder="New playlist title">
//             </div>
//             <button>Create</button>
//         </form>
//         <br>

//         {playlists_rendered}

//         <br><br>
//     "#);

//     let settings = context.collection.get_settings();

//     let html = layout(&settings, &body, &settings.title);

//     HttpResponse::Ok()
//         .content_type(ContentType::html())
//         .body(html)
// }

// pub async fn videos(context: Data<Arc<Context>>) -> HttpResponse {
//     let videos = context.collection.all_videos();
//     let videos_rendered = videos
//         .iter()
//         .map(|video| {
//             let info = match &video.assets.metadata {
//                 Some(metadata) => format!(
//                     "{}×{} {}<br>{} versions",
//                     metadata.width,
//                     metadata.height,
//                     precise_duration(metadata.duration),
//                     video.assets.versions.len()
//                 ),
//                 None => String::from("No source or metadata yet.")
//             };
//             let id = &video.id;
//             let title = &video.title;

//             let poster = video.assets.poster
//                 .as_deref()
//                 .map(|poster| format!(r#"<img class="thumbnail" src="/poster/{poster}">"#))
//                 .unwrap_or(String::new());

//             let unlisted = match video.unlisted {
//                 true => r#" <span class="badge">Unlisted</span>"#,
//                 false => ""
//             };

//             // TODO: Escape data-search, add more in there (description etc.), consider better implementation too of course
//             formatdoc!(r#"
//                 <div class="video" data-id="{id}" data-search="{title}">
//                     {poster}
//                     <div>
//                         <a href="/video/{id}">{title}</a>{unlisted}<br>
//                         {info}
//                     </div>
//                 </div>
//             "#)
//         })
//         .collect::<Vec<String>>()
//         .join("");

//     let video_upload_button = upload_widget("/video", "Upload Video");

//     let controls_rendered = controls();

//     let videos_js = include_str!("../assets/videos.js");

//     let body = formatdoc!(r#"
//         <div class="updates"></div>

//         {controls_rendered}

//         <h2>Videos</h2>

//         {video_upload_button}

//         <input class="filter" placeholder="Filter videos" type="search">

//         <script>
//             {videos_js}
//         </script>

//         <div class="videos">
//             {videos_rendered}
//         </div>
//     "#);

//     let settings = context.collection.get_settings();
//     let html = layout(&settings, &body, &settings.title);

//     HttpResponse::Ok()
//         .content_type(ContentType::html())
//         .body(html)
// }

pub async fn index(context: Data<Arc<Context>>) -> HttpResponse {
    let site = &context.site.read().unwrap();

    let extra = match &site.content {
        SiteContent::Collection(collection) => {
            "What here? (TODO)".to_string()
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
        SiteContent::Playlist(playlist) => {
            let videos_rendered = playlist.videos_physical
                .iter()
                .map(|video| {
                    let mut info = String::new();
                    for version in &video.versions {
                        let compiled = match &version.video_meta {
                            Some(video_meta) => format!(
                                "{}×{} {}<br>",
                                video_meta.width,
                                video_meta.height,
                                precise_duration(video_meta.duration)
                            ),
                            None => String::from("No source or metadata yet.")
                        };

                        info.push_str(&compiled);
                    };
                    let path = &video.path;
                    let title = video.title.as_deref().unwrap_or(video.slug());

                    let poster = if let Some(poster) = &video.poster {
                        if let Some(assets) = &poster.assets {
                            // TODO: Build img with srcset etc.
                            let width = assets.all().first().unwrap().width;
                            format!(r#"<img class="thumbnail" src="cover_{width}.jpg">"#)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    let unlisted = match video.unlisted {
                        true => r#" <span class="badge">Unlisted</span>"#,
                        false => ""
                    };

                    // TODO: Escape data-search, add more in there (description etc.), consider better implementation too of course
                    formatdoc!(r#"
                        <div class="video" data-path="{path}" data-search="{title}">
                            {poster}
                            <div>
                                <a href="/video/{path}">{title}</a>{unlisted}<br>
                                {info}
                            </div>
                        </div>
                    "#)
                })
                .collect::<Vec<String>>()
                .join("");

            let video_upload_button = upload_widget("/video", "Upload Video");
            let videos_js = include_str!("../assets/videos.js");

            formatdoc!(r#"
                <h2>Videos</h2>

                {video_upload_button}

                <input class="filter" placeholder="Filter videos" type="search">

                <script>
                    {videos_js}
                </script>

                <div class="videos">
                    {videos_rendered}
                </div>
            "#)
        }
        SiteContent::Video(video) => {
            let mut info = String::new();
            for version in &video.versions {
                let compiled = match &version.video_meta {
                    Some(video_meta) => format!(
                        "{}×{} {}<br>",
                        video_meta.width,
                        video_meta.height,
                        precise_duration(video_meta.duration)
                    ),
                    None => String::from("No source or metadata yet.")
                };

                info.push_str(&compiled);
            };
            let title = video.title.as_deref().unwrap_or(video.slug());

            let poster = if let Some(poster) = &video.poster {
                if let Some(assets) = &poster.assets {
                    // TODO: Build img with srcset etc.
                    let width = assets.all().first().unwrap().width;
                    format!(r#"<img class="thumbnail" src="cover_{width}.jpg">"#)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let unlisted = match video.unlisted {
                true => r#" <span class="badge">Unlisted</span>"#,
                false => ""
            };

            formatdoc!(r#"
                <h2>Site:Video</h2>

                <div class="videos">
                    <div class="video">
                        {poster}
                        <div>
                            <span>{title}</span>{unlisted}<br>
                            {info}
                        </div>
                    </div>
                </div>
            "#)
        }
    };

    let body = formatdoc!(r#"
        <div class="updates"></div>

        {extra}
    "#);

    let html = layout(&context, "", site, &body, site.title());

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

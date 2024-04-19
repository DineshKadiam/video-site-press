use std::path::Path;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Field,
    Playlist,
    Site,
    SiteContent,
    Video
};
use crate::icons;

fn collection_tree(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    context: &Context,
    path: &str
) -> String {
    let collection_icon = icons::collection(&context.editor_translations.collection);
    let playlist_icon = icons::playlist(&context.editor_translations.playlist);
    let video_icon = icons::video(&context.editor_translations.video);
    let mut subcollections_sorted: Vec<Arc<Collection>> = collection.subcollections.to_vec();

    subcollections_sorted.sort_unstable_by(|a, b| {
        let a_criteria = a.title.as_deref().unwrap_or_else(|| a.slug());
        let b_criteria = b.title.as_deref().unwrap_or_else(|| b.slug());

        a_criteria.cmp(b_criteria)
    });

    let subcollections = subcollections_sorted
        .iter()
        .map(|collection| {
            let collection_path = &collection.path;

            let active = path == collection_path;
            let href = format!("/collection/{collection_path}");
            let node = tree_node(active, context, &collection.errors, &href, &collection_icon, collection.slug(), collection.title.as_deref());

            if active || path
                .strip_prefix(&collection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = collection.title.as_deref().unwrap_or_else(|| collection.slug());
                let (icon, space) = if active { (collection_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{label}</a>"#));
            }

            let collection_content = collection_tree(breadcrumbs, collection, context, path);

            formatdoc!(r#"
                <li>
                    {node}
                    {collection_content}
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let mut playlists_sorted: Vec<Arc<Playlist>> = collection.playlists.to_vec();

    playlists_sorted.sort_unstable_by(|a, b| {
        let a_criteria = a.title.as_deref().unwrap_or_else(|| a.slug());
        let b_criteria = b.title.as_deref().unwrap_or_else(|| b.slug());

        a_criteria.cmp(b_criteria)
    });

    let playlists = playlists_sorted
        .iter()
        .map(|playlist| {
            let playlist_path = &playlist.path;

            let active = path == playlist_path;
            let href = format!("/playlist/{playlist_path}");
            let node = tree_node(active, context, &playlist.errors, &href, &playlist_icon, playlist.slug(), playlist.title.as_deref());

            if active || path
                .strip_prefix(&playlist.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
                let (icon, space) = if active { (playlist_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{label}</a>"#));
            }

            let children = if playlist.videos.is_empty() {
                String::new()
            } else {
                let lis = playlist.videos
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let active = path == video_path;
                        let href = format!("/video/{video_path}");
                        let node = tree_node(active, context, &video.errors, &href, &video_icon, video.slug(), video.title.as_deref());

                        if active {
                            let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                            breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {label}</a>"#));
                        }

                        formatdoc!(r#"
                            <li>
                                {node}
                            </li>
                        "#)
                    })
                    .collect::<Vec<String>>()
                    .join("");

                formatdoc!(r#"
                    <ul>
                        {lis}
                    </ul>
                "#)
            };

            formatdoc!(r#"
                <li>
                    {node}
                    {children}
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let mut videos_sorted: Vec<Arc<Video>> = collection.videos.to_vec();

    videos_sorted.sort_unstable_by(|a, b| {
        let a_criteria = a.title.as_deref().unwrap_or_else(|| a.slug());
        let b_criteria = b.title.as_deref().unwrap_or_else(|| b.slug());

        a_criteria.cmp(b_criteria)
    });

    let videos = videos_sorted
        .iter()
        .map(|video| {
            let video_path = &video.path;

            let active = path == video_path;
            let href = format!("/video/{video_path}");
            let node = tree_node(active, context, &video.errors, &href, &video_icon, video.slug(), video.title.as_deref());

            if active {
                let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {label}</a>"#));
            }

            formatdoc!(r#"
                <li>
                    {node}
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    formatdoc!(r#"
        <ul>
            {subcollections}
            {playlists}
            {videos}
        </ul>
    "#)
}

pub fn form_checkbox(checked: bool, key: &str, label: &str) -> String {
    let checked_attribute = if checked { "checked " } else { "" };

    formatdoc!(r#"
        <div class="form_field">
            <div>
                <input {checked_attribute}id="{key}" name="{key}" type="checkbox">
                <label for="{key}">{label}</label>
            </div>
        </div>
    "#)
}

pub fn form_field(field: &Field, key: &str, label: &str) -> String {
    let error = field.error
        .as_deref()
        .map(|message| format!(r#"<span class="error">{message}</span>"#))
        .unwrap_or(String::new());
    let value = &field.value;

    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <input id="{key}" name="{key}" value="{value}">
            {error}
        </div>
    "#)
}

pub fn form_field_date(
    field: &Field,
    key: &str,
    label: &str
) -> String {
    let error = field.error
        .as_deref()
        .map(|message| format!(r#"<span class="error">{message}</span>"#))
        .unwrap_or(String::new());
    let value = &field.value;

    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <input id="{key}" name="{key}" type="date" value="{value}">
            {error}
        </div>
    "#)
}

pub fn form_field_pattern(
    field: &Field,
    key: &str,
    label: &str,
    placeholder: &str,
    pattern: &str
) -> String {
    let error = field.error
        .as_deref()
        .map(|message| format!(r#"<span class="error">{message}</span>"#))
        .unwrap_or(String::new());
    let value = &field.value;

    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <input id="{key}" name="{key}" placeholder="{placeholder}" pattern="{pattern}" value="{value}">
            {error}
        </div>
    "#)
}

pub fn form_textarea(
    key: &str,
    label: &str,
    value: Option<&str>
) -> String {
    let value = value.unwrap_or("");
    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <textarea id="{key}" name="{key}">{value}</textarea>
        </div>
    "#)
}

pub fn layout(
    context: &Context,
    path: &str,
    site: &Site,
    body: &str,
    title: &str
) -> String {
    let (breadcrumbs, tree) = site_tree(context, path, site, &context.site_dir);

    // TODO: Possibly hash scripts/styles
    let head = formatdoc!(r#"
        <head>
            <title>{title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="/favicon.png" rel="icon" type="image/png">
            <!-- <link href="favicon_light.png" rel="icon" type="image/png" media="(prefers-color-scheme: light)"> -->
            <script defer src="/scripts.js"></script>
            <link rel="stylesheet" href="/styles.css">
        </head>
    "#);

    let hyper8_version = env!("CARGO_PKG_VERSION");

    let external_icon = icons::external(&context.editor_translations.external_link);
    let t_close = &context.editor_translations.close;
    let splash = formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <img src="/splash.jpg?{hyper8_version}">
                <div class="pad">
                    <ul autofocus>
                        <li>Navigate the site's content in the left sidebar</li>
                        <li>Make changes in the main area of the page</li>
                        <li>Use the controls top-right to preview/deploy</li>
                    </ul>
                    <div class="hsplit">
                        <a href="https://simonrepp.com/hyper8" target="_blank">
                            {external_icon}
                            <span>Hyper 8 Video System {hyper8_version}</span>
                        </a>
                        <form method="dialog">
                            <button autofocus>{t_close}</button>
                        </form>
                    </div>
                </div>
            </div>
        </dialog>
    "#);

    let t_deploy = &context.editor_translations.deploy;
    let t_deploy_configuration_missing_required_fields = &context.editor_translations.deploy_configuration_missing_required_fields;
    let t_really_deploy = &context.editor_translations.really_deploy;
    let deploy_button = match site.deploy_config.complete() {
        true => format!(r#"<form action="/deploy" method="post" onsubmit="return confirm('{t_really_deploy}');"><button>{t_deploy}</button></form>"#),
        // TODO: Instead of disabled link, offer "Configure Deployment" link
        // (Site form + hash that goes directly to the relevant section, or dedicated page to clean up interface by area of concern?)
        false => format!(r#"<button disabled title="{t_deploy_configuration_missing_required_fields}">{t_deploy}*</button>"#)
    };

    let logo_icon = icons::logo(&context.editor_translations.hyper8_logo);
    let t_filter = &context.editor_translations.filter;
    let t_preview = &context.editor_translations.preview;
    let body = formatdoc!(r#"
            <body data-version="{hyper8_version}">
                <div class="controls">
                    <div class="buttons">
                        <form action="/build" method="post" target="_blank"><button>{t_preview}</button></form>
                        {deploy_button}
                        <button id="toggle-splash">{logo_icon}</button>
                    </div>
                    <span class="processing"></span>
                </div>
                <div class="split">
                    <nav>
                        <input id="filter_tree" placeholder="{t_filter}" type="search">
                        {tree}
                    </nav>
                    <main>
                        <div class="breadcrumbs">
                            {breadcrumbs}
                        </div>
                        {body}
                    </main>
                </div>
                {splash}
            </body>
        "#
    );

    formatdoc!(r#"
        <!doctype html>
        <html lang="en">
            {head}
            {body}
        </html>
    "#)
}

pub fn not_found(context: &Context, message: &str) -> HttpResponse {
    let t_back = &context.editor_translations.back;
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="/">{t_back}</a>
    "#);

    let site = &context.get_site();
    let html = layout(context, "", site, &body, "Not Found");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn site_tree(
    context: &Context,
    path: &str,
    site: &Site,
    site_dir: &Path
) -> (String, String) {
    let mut breadcrumbs = Vec::new();

    let root_active = path == "";
    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let t_collection = &context.editor_translations.collection;
            let collection_icon = icons::collection(t_collection);
            let breadcrumb_title = collection.title.as_deref().unwrap_or(t_collection);
            let (icon, space) = if root_active { (collection_icon.as_str(), " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/collection/">{icon}{space}{breadcrumb_title}</a>"#));

            let collection_content = collection_tree(&mut breadcrumbs, collection, context, path);

            let errors = if collection.errors.is_empty() { &site.errors } else { &collection.errors };
            let node = tree_node(root_active, context, errors, "/collection/", &collection_icon, site_dir, collection.title.as_deref());

            formatdoc!(r#"
                {node}
                {collection_content}
            "#)
        }
        SiteContent::Empty |
        SiteContent::Invalid => {
            let t_empty_site = &context.editor_translations.empty_site;
            // TODO: Dedicated icon for empty site?
            let collection_icon = icons::collection(t_empty_site);

            breadcrumbs.push(format!(r#"<a href="/">{root_dir}</a>"#));

            tree_node(root_active, context, &Vec::new(), "/", &collection_icon, site_dir, Some(t_empty_site))
        }
        SiteContent::Playlist(playlist) => {
            let t_playlist = &context.editor_translations.playlist;
            let playlist_icon = icons::playlist(t_playlist);
            let breadcrumb_title = playlist.title.as_deref().unwrap_or(t_playlist);
            let (icon, space) = if root_active { (playlist_icon.as_str(), " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/playlist/">{icon}{space}{breadcrumb_title}</a>"#));

            let videos = if playlist.videos.is_empty() {
                formatdoc!(r#"
                    <li>
                        <span class="muted">Empty</span>
                    </li>
                "#)
            } else {
                let video_icon = icons::video(&context.editor_translations.video);

                let mut videos_sorted: Vec<Arc<Video>> = playlist.videos.to_vec();

                videos_sorted.sort_unstable_by(|a, b| {
                    let a_criteria = a.title.as_deref().unwrap_or_else(|| a.slug());
                    let b_criteria = b.title.as_deref().unwrap_or_else(|| b.slug());

                    a_criteria.cmp(b_criteria)
                });

                videos_sorted
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let active = path == video_path;
                        let href = format!("/video/{video_path}");
                        let slug = video.slug();

                        if active {
                            breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {slug}</a>"#));
                        }

                        let node = tree_node(active, context, &video.errors, &href, &video_icon, slug, video.title.as_deref());

                        formatdoc!(r#"
                            <li>
                                {node}
                            </li>
                        "#)
                    })
                    .collect::<Vec<String>>()
                    .join("")
            };

            let node = tree_node(root_active, context, &playlist.errors, "/playlist/", &playlist_icon, site_dir, playlist.title.as_deref());

            formatdoc!(r#"
                {node}
                <ul>
                    {videos}
                </ul>
            "#)
        }
        SiteContent::Video(video) => {
            let t_video = &context.editor_translations.video;
            let video_icon = icons::video(t_video);

            let breadcrumb_title = video.title.as_deref().unwrap_or(t_video);
            let (icon, space) = if root_active { (video_icon.as_str(), " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/video/">{icon}{space}{breadcrumb_title}</a>"#));

            tree_node(root_active, context, &video.errors, "/video/", &video_icon, site_dir, video.title.as_deref())
        }
    };

    let breadcrumbs = breadcrumbs.join(" › ");

    let sidebar = formatdoc!(r#"
            {site_content}
    "#);

    (breadcrumbs, sidebar)
}

fn tree_node(
    active: bool,
    context: &Context,
    errors: &[String],
    href: &str,
    icon: &str,
    slug: &str,
    title_option: Option<&str>
) -> String {
    let active = if active { r#" class="active""# } else { "" };
    let icon_container = if errors.is_empty() {
        icon.to_string()
    } else {
        format!(r#"
            <span style="position: relative;">
                <span class="errors_badge"></span>
                {icon}
            </span>
        "#)
    };
    let t_no_title = &context.editor_translations.no_title;
    let title = title_option
        .map(|title| title.to_string())
        .unwrap_or(format!(r#"<span class="muted">{t_no_title}</span>"#));

    let depth = if href.ends_with('/') { 0 } else { href.split('/').count() - 2 };
    let padding_left_rem = match depth {
        0 => 0.4,
        1 => 1.4,
        other => other as f32 + 0.6
    };
    let padding_left = format!("{}rem", padding_left_rem);

    formatdoc!(r#"
        <a{active} href="{href}" style="padding-left: {padding_left};">
            {icon_container}
            <div class="labels">
                <span class="title">{title}</span>
                <span class="path">{slug}</span>
            </div>
        </a>
    "#)
}

pub fn upload_widget(action: &str, label: &str) -> String {
    formatdoc!(r#"
        <form action="{action}" method="post" enctype="multipart/form-data">
            <input name="file" type="file">
            <button class="upload_button">{label}</button>
        </form>
    "#)
}
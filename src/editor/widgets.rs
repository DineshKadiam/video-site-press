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

const ICON_COLLECTION: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m9.8672 9.9004v19.602h19.602v-19.602h-19.602zm24.664 0v19.602h19.602v-19.602h-19.602zm-24.664 24.664v19.602h19.602v-19.602h-19.602zm24.664 0v19.602h19.602v-19.602h-19.602z"/>
</svg>
"#;

const ICON_PLAYLIST: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m32 7.4073c-13.582 6.153e-4 -24.592 11.011-24.593 24.593 6.153e-4 13.582 11.011 24.592 24.593 24.593 13.582-6.12e-4 24.592-11.011 24.593-24.593-6.12e-4 -13.582-11.011-24.592-24.593-24.593zm0 6.3023c10.101 4.56e-4 18.29 8.1893 18.29 18.29-4.56e-4 10.101-8.1893 18.29-18.29 18.29-10.101-4.56e-4 -18.29-8.1893-18.29-18.29 4.56e-4 -10.101 8.1893-18.29 18.29-18.29zm-6.5521 7.515 0.02539 21.551 17.973-10.451z"/>
</svg>
"#;

const ICON_VIDEO: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m9.8672 9.8672v44.266h44.266v-44.266h-44.266zm5.0625 4.4121h5.0625v5.0625h-5.0625v-5.0625zm29.078 0h5.0625v5.0625h-5.0625v-5.0625zm-29.078 10.125h5.0625v5.0645h-5.0625v-5.0645zm29.078 0h5.0625v5.0645h-5.0625v-5.0645zm-29.078 10.127h5.0625v5.0645h-5.0625v-5.0645zm29.078 0h5.0625v5.0645h-5.0625v-5.0645zm-29.078 10.127h5.0625v5.0625h-5.0625v-5.0625zm29.078 0h5.0625v5.0625h-5.0625v-5.0625z"/>
</svg>
"#;

fn collection_tree(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    context: &Context,
    path: &str
) -> String {
    let mut subcollections_sorted: Vec<Arc<Collection>> = collection.subcollections.iter().cloned().collect();

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
            let node = tree_node(active, &collection.errors, &href, ICON_COLLECTION, collection.slug(), collection.title.as_deref());

            if active || path
                .strip_prefix(&collection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = collection.title.as_deref().unwrap_or_else(|| collection.slug());
                breadcrumbs.push(format!(r#"<a href="{href}">{ICON_COLLECTION} {label}</a>"#));
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

    let mut playlists_sorted: Vec<Arc<Playlist>> = collection.playlists.iter().cloned().collect();

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
            let node = tree_node(active, &playlist.errors, &href, ICON_PLAYLIST, playlist.slug(), playlist.title.as_deref());

            if active || path
                .strip_prefix(&playlist.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
                breadcrumbs.push(format!(r#"<a href="{href}">{ICON_PLAYLIST} {label}</a>"#));
            }

            let children = if playlist.videos_physical.is_empty() {
                String::new()
            } else {
                let lis = playlist.videos_physical
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let active = path == video_path;
                        let href = format!("/video/{video_path}");
                        let node = tree_node(active, &video.errors, &href, ICON_VIDEO, video.slug(), video.title.as_deref());

                        if active {
                            let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                            breadcrumbs.push(format!(r#"<a href="{href}">{ICON_VIDEO} {label}</a>"#));
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

    let mut videos_sorted: Vec<Arc<Video>> = collection.videos.iter().cloned().collect();

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
            let node = tree_node(active, &video.errors, &href, ICON_VIDEO, video.slug(), video.title.as_deref());

            if active {
                let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                breadcrumbs.push(format!(r#"<a href="{href}">{ICON_VIDEO} {label}</a>"#));
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

pub fn form_checkbox(
    checked: bool,
    key: &str,
    label: &str
) -> String {
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

pub fn form_field(
    field: &Field,
    key: &str,
    label: &str,
    placeholder: &str
) -> String {
    let error = field.error
        .as_deref()
        .map(|message| format!(r#"<span class="error">{message}</span>"#))
        .unwrap_or(String::new());
    let value = &field.value;

    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <input id="{key}" name="{key}" placeholder="{placeholder}" value="{value}">
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
    placeholder: &str,
    value: Option<&str>
) -> String {
    let value = value.unwrap_or("");
    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <textarea id="{key}" name="{key}" placeholder="{placeholder}">{value}</textarea>
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
    let (breadcrumbs, sidebar) = site_tree(&context, path);

    // TODO: Possibly hash scripts/styles
    let head = formatdoc!(r#"
        <head>
            <title>{title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="favicon.png" rel="icon" type="image/png">
            <!-- <link href="favicon_light.png" rel="icon" type="image/png" media="(prefers-color-scheme: light)"> -->
            <script defer src="/scripts.js"></script>
            <link rel="stylesheet" href="/styles.css">
        </head>
    "#);

    let body = formatdoc!(r#"
            <body>
                <script>document.body.classList.add('js_enabled');</script>
                <header>
                    <a class="collection_title" href="/">{title}</a>
                    <div class="controls">
                        <form action="/build" method="post" target="_blank"><button>Preview</button></form>
                        <form action="/deploy" method="post"><button>Deploy</button></form>
                    </div>
                </header>
                <div class="split">
                    {sidebar}
                    <main>
                        <div class="breadcrumbs">
                            {breadcrumbs}
                        </div>
                        {body}
                    </main>
                </div>
                <footer>
                    Hyper 8 Alpha
                </footer>
            </body>
        "#,
        title = site.title()
    );

    format!("<!doctype html>\n<html>{head}{body}</html>")
}

pub fn not_found(context: &Context, message: &str) -> HttpResponse {
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="/">Back to index</a>
    "#);

    let site = &context.site.read().unwrap();
    let html = layout(context, "", site, &body, "Not Found");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn site_tree(context: &Context, path: &str) -> (String, String) {
    let site: &Site = &context.site.read().unwrap();

    let mut breadcrumbs = Vec::new();

    let root_dir = context.site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &context.site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let breadcrumb_title = collection.title.as_deref().unwrap_or("Collection");
            breadcrumbs.push(format!(r#"<a href="/collection/">{ICON_COLLECTION} {breadcrumb_title}</a>"#));

            let collection_content = collection_tree(&mut breadcrumbs, collection, &context, path);

            let active = path == "";
            let errors = if collection.errors.is_empty() { &site.errors } else { &collection.errors };
            let node = tree_node(active, errors, "/collection/", ICON_COLLECTION, site_dir, collection.title.as_deref());

            formatdoc!(r#"
                {node}
                {collection_content}
            "#)
        }
        SiteContent::Empty |
        SiteContent::Invalid => {
            breadcrumbs.push(format!(r#"<a href="/">{root_dir}</a>"#));

            let active = path == "";
            // TODO: Dedicated icon for empty site?
            tree_node(active, &Vec::new(), "/", ICON_COLLECTION, site_dir, Some("Empty site"))
        }
        SiteContent::Playlist(playlist) => {
            let breadcrumb_title = playlist.title.as_deref().unwrap_or("Playlist");
            breadcrumbs.push(format!(r#"<a href="/playlist/">{ICON_PLAYLIST} {breadcrumb_title}</a>"#));

            let videos = if playlist.videos_physical.is_empty() {
                formatdoc!(r#"
                    <li>
                        <span class="muted">Empty</span>
                    </li>
                "#)
            } else {
                let mut videos_sorted: Vec<Arc<Video>> = playlist.videos_physical.iter().cloned().collect();

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
                            breadcrumbs.push(format!(r#"<a href="{href}">{ICON_VIDEO} {slug}</a>"#));
                        }

                        let node = tree_node(active, &video.errors, &href, ICON_VIDEO, slug, video.title.as_deref());

                        formatdoc!(r#"
                            <li>
                                {node}
                            </li>
                        "#)
                    })
                    .collect::<Vec<String>>()
                    .join("")
            };

            let active = path == "";
            let node = tree_node(active, &playlist.errors, "/playlist/", ICON_PLAYLIST, site_dir, playlist.title.as_deref());

            formatdoc!(r#"
                {node}
                <ul>
                    {videos}
                </ul>
            "#)
        }
        SiteContent::Video(video) => {
            let breadcrumb_title = video.title.as_deref().unwrap_or("Video");
            breadcrumbs.push(format!(r#"<a href="/video/">{ICON_VIDEO} {breadcrumb_title}</a>"#));

            let active = path == "";
            tree_node(active, &video.errors, "/video/", ICON_VIDEO, site_dir, video.title.as_deref())
        }
    };

    let breadcrumbs = breadcrumbs.join(" › ");

    let sidebar = formatdoc!(r#"
        <nav>
            {site_content}
        </nav>
    "#);

    (breadcrumbs, sidebar)
}

fn tree_node(
    active: bool,
    errors: &Vec<String>,
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
    let title = title_option.unwrap_or(r#"<span class="muted">No title</span>"#);

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
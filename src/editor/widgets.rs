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

const ICON_COLLECTION: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m9.8672 9.9004v19.602h19.602v-19.602h-19.602zm24.664 0v19.602h19.602v-19.602h-19.602zm-24.664 24.664v19.602h19.602v-19.602h-19.602zm24.664 0v19.602h19.602v-19.602h-19.602z"/>
</svg>
"#;

const ICON_EXTERNAL: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m55.258 55.088v-21.334h-6.2832v15.074h-33.691v-33.697h15.066v-6.2832h-21.35v46.266zm0-46.24h-20.184v6.2832h9.6582l-19.932 19.932 4.4434 4.4453 19.73-19.73v9.2793h6.2832z"/>
</svg>
"#;

const ICON_LOGO: &'static str = r#"
<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
    <path d="m32 2.1322c-0.94981 3.54e-5 -1.8871 0.11116-2.801 0.31056-14.116 1.3254-25.371 12.566-26.735 26.679-0.4247 1.9008-0.4247 3.8723 0 5.7731 1.3655 14.126 12.639 25.372 26.772 26.681 1.8217 0.38869 3.7069 0.38873 5.5285 0 14.133-1.3097 25.406-12.555 26.771-26.681 0.4247-1.9008 0.4247-3.8723 0-5.7731-1.3643-14.114-12.619-25.354-26.735-26.679-0.91412-0.19938-1.851-0.31064-2.801-0.31056zm0 2.6465c2.7864-2.461e-4 5.4576 1.1067 7.4279 3.0771 4.1132 4.1128 4.1132 10.741 0 14.854l-7.4279 7.4264-7.4264-7.4264c-4.1132-4.1128-4.1132-10.741 0-14.854 1.97-1.97 4.6404-3.077 7.4264-3.0771zm-9.6138 1.5348c-4.8054 5.147-4.702 13.252 0.31506 18.269l7.4264 7.4264-7.4264 7.4264c-5.0171 5.0166-5.1205 13.122-0.31506 18.269-7.419-2.7784-13.317-8.6618-16.086-16.091 2.8443 2.6588 6.5904 3.8211 10.223 3.4776 0.22268-0.95843 0.5352-1.8994 0.93618-2.81-3.359 0.7149-6.9943-0.2281-9.6123-2.846-4.1127-4.1127-4.1127-10.74 0-14.853 2.0564-2.0564 4.7412-3.0846 7.4264-3.0846 0.73327 0 1.4653 0.0852 2.1859 0.23854-0.40097-0.91065-0.71349-1.8516-0.93618-2.81-0.41549-0.03928-0.83271-0.06001-1.2497-0.06001-3.2286 0-6.4544 1.1806-8.9732 3.5347 2.7699-7.4278 8.6672-13.31 16.086-16.087zm19.228 0c7.4189 2.7782 13.316 8.6607 16.086 16.089-2.5189-2.3547-5.7457-3.5362-8.9747-3.5362-0.41656 0-0.83321 0.02082-1.2482 0.06001-0.22261 0.95812-0.53686 1.8982-0.93768 2.8085 0.72036-0.15334 1.4526-0.23704 2.1859-0.23704 2.6852 0 5.37 1.0282 7.4264 3.0846 4.1127 4.1128 4.1127 10.74 0 14.853-2.6177 2.6176-6.2526 3.5619-9.6108 2.8475 0.40066 0.9102 0.71364 1.8506 0.93618 2.8085 3.6326 0.343 7.3804-0.81841 10.224-3.4776-2.7694 7.4291-8.6677 13.312-16.088 16.091 4.8057-5.147 4.7037-13.252-0.31356-18.269l-7.4279-7.4264 7.4279-7.4264c5.0172-5.0167 5.1193-13.122 0.31356-18.269zm-9.6138 27.568 7.4279 7.4264c4.1132 4.1128 4.1132 10.741 0 14.854-4.1129 4.1132-10.741 4.1132-14.854 0-4.1132-4.1128-4.1132-10.741 0-14.854z"/>
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
    path: &str
) -> String {
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
            let node = tree_node(active, &collection.errors, &href, ICON_COLLECTION, collection.slug(), collection.title.as_deref());

            if active || path
                .strip_prefix(&collection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = collection.title.as_deref().unwrap_or_else(|| collection.slug());
                let (icon, space) = if active { (ICON_COLLECTION, " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{label}</a>"#));
            }

            let collection_content = collection_tree(breadcrumbs, collection, path);

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
            let node = tree_node(active, &playlist.errors, &href, ICON_PLAYLIST, playlist.slug(), playlist.title.as_deref());

            if active || path
                .strip_prefix(&playlist.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                let label = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
                let (icon, space) = if active { (ICON_PLAYLIST, " ") } else { ("", "") };
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
    let (breadcrumbs, tree) = site_tree(path, site, &context.site_dir);

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
                            {ICON_EXTERNAL}
                            <span>Hyper 8 Video System {hyper8_version}</span>
                        </a>
                        <form method="dialog">
                            <button autofocus>Close</button>
                        </form>
                    </div>
                </div>
            </div>
        </dialog>
    "#);

    let deploy_button = match site.deploy_config.complete() {
        true => r#"<form action="/deploy" method="post"><button>Deploy</button></form>"#,
        // TODO: Instead of disabled link, offer "Configure Deployment" link
        // (Site form + hash that goes directly to the relevant section, or dedicated page to clean up interface by area of concern?)
        false => r#"<button disabled title="Deploy configuration is missing required fields">Deploy*</button>"#
    };

    let body = formatdoc!(r#"
            <body data-version="{hyper8_version}">
                <div class="controls">
                    <div class="buttons">
                        <form action="/build" method="post" target="_blank"><button>Preview</button></form>
                        {deploy_button}
                        <button id="toggle-splash">{ICON_LOGO}</button>
                    </div>
                    <span class="processing"></span>
                </div>
                <div class="split">
                    <nav>
                        <input id="filter_tree" placeholder="Filter" type="search">
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
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="/">Back to index</a>
    "#);

    let site = &context.get_site();
    let html = layout(context, "", site, &body, "Not Found");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn site_tree(path: &str, site: &Site, site_dir: &Path) -> (String, String) {
    let mut breadcrumbs = Vec::new();

    let root_active = path == "";
    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let breadcrumb_title = collection.title.as_deref().unwrap_or("Collection");
            let (icon, space) = if root_active { (ICON_COLLECTION, " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/collection/">{icon}{space}{breadcrumb_title}</a>"#));

            let collection_content = collection_tree(&mut breadcrumbs, collection, path);

            let errors = if collection.errors.is_empty() { &site.errors } else { &collection.errors };
            let node = tree_node(root_active, errors, "/collection/", ICON_COLLECTION, site_dir, collection.title.as_deref());

            formatdoc!(r#"
                {node}
                {collection_content}
            "#)
        }
        SiteContent::Empty |
        SiteContent::Invalid => {
            breadcrumbs.push(format!(r#"<a href="/">{root_dir}</a>"#));

            // TODO: Dedicated icon for empty site?
            tree_node(root_active, &Vec::new(), "/", ICON_COLLECTION, site_dir, Some("Empty site"))
        }
        SiteContent::Playlist(playlist) => {
            let breadcrumb_title = playlist.title.as_deref().unwrap_or("Playlist");
            let (icon, space) = if root_active { (ICON_PLAYLIST, " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/playlist/">{icon}{space}{breadcrumb_title}</a>"#));

            let videos = if playlist.videos.is_empty() {
                formatdoc!(r#"
                    <li>
                        <span class="muted">Empty</span>
                    </li>
                "#)
            } else {
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

            let node = tree_node(root_active, &playlist.errors, "/playlist/", ICON_PLAYLIST, site_dir, playlist.title.as_deref());

            formatdoc!(r#"
                {node}
                <ul>
                    {videos}
                </ul>
            "#)
        }
        SiteContent::Video(video) => {
            let breadcrumb_title = video.title.as_deref().unwrap_or("Video");
            let (icon, space) = if root_active { (ICON_VIDEO, " ") } else { ("", "") };
            breadcrumbs.push(format!(r#"<a href="/video/">{icon}{space}{breadcrumb_title}</a>"#));

            tree_node(root_active, &video.errors, "/video/", ICON_VIDEO, site_dir, video.title.as_deref())
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
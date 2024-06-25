// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Language,
    Site,
    SiteContent,
    Translations,
    TreeViewOrder
};
use crate::icons;
use crate::util::{
    html_escape_outside_attribute,
    human_size,
    precise_duration
};

mod links;
mod splash_screen;

pub use links::{add_link_form, links};
use splash_screen::splash_screen;

fn collection_tree(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    context: &Context,
    path: Option<&str>,
    translations: &Translations,
    tree_view_order: &TreeViewOrder
) -> String {
    let collection_icon = icons::collection(translations.collection);
    let playlist_icon = icons::playlist(translations.playlist);
    let video_icon = icons::video(translations.video);
    let subcollections = collection.subcollections_in_tree_view_order(tree_view_order)
        .iter()
        .map(|collection| {
            let collection_path = &collection.path;

            let active = path.is_some_and(|path| path == collection_path);
            let href = format!("/collection/{collection_path}");
            let node = tree_node(
                active,
                &collection.errors,
                &href,
                &collection_icon,
                collection.slug(),
                collection.title.as_deref(),
                translations,
                collection.unlisted
            );

            if active || path.is_some_and(|path|
                path
                    .strip_prefix(&collection.path)
                    .is_some_and(|rest| rest.starts_with('/'))
            ) {
                let label = collection.title.as_deref().unwrap_or_else(|| collection.slug());
                let e_label = html_escape_outside_attribute(label);
                let (icon, space) = if active { (collection_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{e_label}</a>"#));
            }

            let collection_content = collection_tree(breadcrumbs, collection, context, path, translations, tree_view_order);

            formatdoc!(r#"
                <li>
                    {node}
                    {collection_content}
                </li>
            "#)
        })
        .collect::<Vec<String>>()
        .join("");

    let playlists = collection.playlists_in_tree_view_order(tree_view_order)
        .iter()
        .map(|playlist| {
            let playlist_path = &playlist.path;

            let active = path.is_some_and(|path| path == playlist_path);
            let href = format!("/playlist/{playlist_path}");
            let node = tree_node(
                active,
                &playlist.errors,
                &href,
                &playlist_icon,
                playlist.slug(),
                playlist.title.as_deref(),
                translations,
                playlist.unlisted
            );

            if active || path.is_some_and(|path| 
                path
                    .strip_prefix(&playlist.path)
                    .is_some_and(|rest| rest.starts_with('/'))
            ) {
                let label = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
                let e_label = html_escape_outside_attribute(label);
                let (icon, space) = if active { (playlist_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{e_label}</a>"#));
            }

            let children = if playlist.videos.is_empty() {
                String::new()
            } else {
                let lis = playlist.videos_in_tree_view_order(tree_view_order)
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let active = path.is_some_and(|path| path == video_path);
                        let href = format!("/video/{video_path}");
                        let node = tree_node(
                            active,
                            &video.errors,
                            &href,
                            &video_icon,
                            video.slug(),
                            video.title.as_deref(),
                            translations,
                            video.unlisted
                        );

                        if active {
                            let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                            let e_label = html_escape_outside_attribute(label);
                            breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {e_label}</a>"#));
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

    let videos = collection.videos_in_tree_view_order(tree_view_order)
        .iter()
        .map(|video| {
            let video_path = &video.path;

            let active = path.is_some_and(|path| path == video_path);
            let href = format!("/video/{video_path}");
            let node = tree_node(
                active,
                &video.errors,
                &href,
                &video_icon,
                video.slug(),
                video.title.as_deref(),
                translations,
                video.unlisted
            );

            if active {
                let label = video.title.as_deref().unwrap_or_else(|| video.slug());
                let e_label = html_escape_outside_attribute(label);
                breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {e_label}</a>"#));
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

pub fn error_page(
    context: &Context,
    language: &Language,
    message: &str
) -> HttpResponse {
    let t_back = language.translations.back;
    let body = formatdoc!(r#"
        <div>
            {message}
        </div>

        <a href="/">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_error = language.translations.error;
    let html = layout(context, language, None, site, &body, t_error);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

/// When path is Some() that signifies we are viewing some node in the site tree.
/// If it's None we are on some auxillary page such as the dashboard or 404 page.
pub fn layout(
    context: &Context,
    language: &Language,
    path: Option<&str>,
    site: &Site,
    body: &str,
    title: &str
) -> String {
    let translations = &language.translations;
    let (breadcrumbs, tree) = site_tree(context, path, site, &context.site_dir, translations);

    // TODO: Possibly hash scripts/styles
    let e_title = html_escape_outside_attribute(title);
    let head = formatdoc!(r#"
        <head>
            <title>{e_title}</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link href="/favicon.png" rel="icon" type="image/png">
            <!-- <link href="favicon_light.png" rel="icon" type="image/png" media="(prefers-color-scheme: light)"> -->
            <script defer src="/scripts.js"></script>
            <link rel="stylesheet" href="/styles.css">
        </head>
    "#);

    let splash_screen_rendered = splash_screen(language, context.get_editor_theme());

    let t_configure_deployment = translations.configure_deployment;
    let configure_icon = icons::configure(translations.configure);
    let deployment_icon = icons::deployment(translations.deployment);
    let deployment_button = match site.deploy_config.complete() {
        true => {
            let deployment = &context.deploying.lock().unwrap();
            if deployment.active() {
                let t_deploying  = translations.deploying;
                let duration = precise_duration(deployment.started_at.unwrap().elapsed().map(|duration| duration.as_secs_f32()).unwrap_or(0.0));
                formatdoc!(r#"
                    <a href="/deployment">
                        {deployment_icon} {t_deploying} ({duration})
                    </a>
                "#)
            } else {
                let base_url = match &site.base_url {
                    Some(base_url) => format!(" ({base_url})"),
                    None => String::new()
                };
                let t_deploy = translations.deploy;
                let t_really_deploy = translations.really_deploy;
                formatdoc!(r#"
                    <form
                        action="/deploy"
                        data-confirm
                        data-confirm-label="{t_deploy}"
                        data-confirm-text="{t_really_deploy}{base_url}"
                        method="post"">
                        <button>{deployment_icon} {t_deploy}</button>
                    </form>
                "#)
            }
        }
        false => {
            formatdoc!(r#"
                <a href="/configure-deployment">
                    {configure_icon} {t_configure_deployment}
                </a>
            "#)
        }
    };

    let t_processing = translations.processing;
    let processing_icon = icons::processing(t_processing);
    let processing_link = {
        let processing = &context.processing.lock().unwrap();
        if processing.operations.is_empty() {
            let t_idle = translations.idle;
            format!(r#"<a class="processing" href="/jobs">{processing_icon} {t_idle}</a>"#)
        } else {
            let operations = processing.operations
                .iter()
                .map(|operation| {
                    match &operation.last_progress {
                        Some(last_progress) => {
                            let progress = (last_progress.frame as f32 / last_progress.estimated_total_frames as f32).min(1.0);
                            let percent = (progress * 100.0) as usize;
                            let seconds_remaining = (1.0 - progress) * (last_progress.duration / progress);
                            let time_remaining = precise_duration(seconds_remaining);

                            formatdoc!(r#"
                                <div class="processing_badge">
                                    <span>{percent}%</span>
                                    <span class="inverted">-{time_remaining}</span>
                                </div>
                            "#)
                        }
                        None => {
                            formatdoc!(r#"
                                <div class="processing_badge">
                                    MISC
                                </div>
                            "#)
                        }
                    }
                })
                .collect::<Vec<String>>()
                .join("");

            formatdoc!(r#"
                <a class="active processing" href="/jobs">
                    {processing_icon} {operations}
                </a>
            "#)
        }
    };

    let tree_view_order_label = context.get_editor_tree_view_order().key(translations);

    let data_base_url = match &site.base_url {
        Some(base_url) => format!(r#"data-base-url="{base_url}""#),
        None => String::new()
    };
    let storage_usage = human_size(site.storage_usage());
    let hyper8_version = env!("CARGO_PKG_VERSION");
    let logo_icon = icons::logo(translations.hyper8_logo);
    let t_preview = translations.preview;
    let preview_icon = icons::preview(t_preview);
    let storage_icon = icons::storage(translations.storage_usage);
    let t_about = translations.about;
    let t_cancel = translations.cancel;
    let t_filter = translations.filter;
    let body = formatdoc!(r#"
        <body {data_base_url} data-version="{hyper8_version}">
            <div class="controls">
                {processing_link}
                <a href="/">{storage_icon} {storage_usage}</a>
                <form action="/build" method="post" target="_blank"><button>{preview_icon} {t_preview}</button></form>
                <div id="deployment">{deployment_button}</div>
                <button id="toggle-splash">{logo_icon} {t_about}</button>
            </div>
            <div class="split">
                <nav>
                    <div class="tree_view_options">
                        <input id="filter_tree" placeholder="{t_filter}" type="search">
                        <form action="/change-tree-view-order" id="tree-view-order" method="post">
                            <input name="redirect" type="hidden">
                            <button>{tree_view_order_label}</button>
                        </form>
                    </div>
                    {tree}
                </nav>
                <main>
                    {breadcrumbs}
                    {body}
                </main>
            </div>
            {splash_screen_rendered}
            <template id="configure_icon">
                {configure_icon}
            </template>
            <template id="confirmation_dialog">
                <dialog class="confirmation_wrapper">
                    <div class="text"></div>
                    <div class="buttons">
                        <button autofocus class="cancel">{t_cancel}</button>
                        <button class="confirm danger"></button>
                    </div>
                </dialog>
            </template>
            <template id="deployment_icon">
                {deployment_icon}
            </template>
            <template id="processing_icon">
                {processing_icon}
            </template>
        </body>
    "#);

    formatdoc!(r#"
        <!doctype html>
        <html lang="en">
            {head}
            {body}
        </html>
    "#)
}

pub fn not_found(
    context: &Context,
    language: &Language,
    path: &str
) -> HttpResponse {
    let t_back = language.translations.back;
    let t_message = language.translations.the_resource_xxx_was_not_found_message(path);
    let body = formatdoc!(r#"
        <div>
            {t_message}
        </div>

        <a href="/">{t_back}</a>
    "#);

    let site = &context.get_site();

    let t_resource_not_found = language.translations.resource_not_found;
    let html = layout(context, language, None, site, &body, t_resource_not_found);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

pub fn site_tree(
    context: &Context,
    path: Option<&str>,
    site: &Site,
    site_dir: &Path,
    translations: &Translations
) -> (String, String) {
    let mut breadcrumbs = Vec::new();

    let root_active = path.is_some_and(|path| path == "");
    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let t_collection = translations.collection;
            let collection_icon = icons::collection(t_collection);

            if path.is_some() {
                let breadcrumb_title = collection.title.as_deref().unwrap_or(t_collection);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                let (icon, space) = if root_active { (collection_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/collection/">{icon}{space}{e_breadcrumb_title}</a>"#));
            }

            let tree_view_order = context.get_editor_tree_view_order();

            let collection_content = collection_tree(&mut breadcrumbs, collection, context, path, translations, &tree_view_order);

            let errors = if collection.errors.is_empty() { &site.errors } else { &collection.errors };
            let node = tree_node(
                root_active,
                errors,
                "/collection/",
                &collection_icon,
                site_dir,
                collection.title.as_deref(),
                translations,
                collection.unlisted
            );

            formatdoc!(r#"
                {node}
                {collection_content}
            "#)
        }
        SiteContent::Empty |
        SiteContent::Invalid => {
            let t_empty_site = translations.empty_site;
            // TODO: Dedicated icon for empty site?
            let collection_icon = icons::collection(t_empty_site);

            if path.is_some() {
                breadcrumbs.push(format!(r#"<a href="/">{root_dir}</a>"#));
            }

            tree_node(
                root_active,
                &Vec::new(),
                "/",
                &collection_icon,
                site_dir,
                Some(t_empty_site),
                translations,
                false
            )
        }
        SiteContent::Playlist(playlist) => {
            let t_playlist = translations.playlist;
            let playlist_icon = icons::playlist(t_playlist);

            if path.is_some() {
                let breadcrumb_title = playlist.title.as_deref().unwrap_or(t_playlist);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                let (icon, space) = if root_active { (playlist_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/playlist/">{icon}{space}{e_breadcrumb_title}</a>"#));
            }

            let videos = if playlist.videos.is_empty() {
                String::new()
            } else {
                let tree_view_order = context.get_editor_tree_view_order();

                let video_icon = icons::video(translations.video);

                playlist.videos_in_tree_view_order(&tree_view_order)
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let active = path.is_some_and(|path| path == video_path);
                        let href = format!("/video/{video_path}");
                        let slug = video.slug();

                        if active {
                            breadcrumbs.push(format!(r#"<a href="{href}">{video_icon} {slug}</a>"#));
                        }

                        let node = tree_node(
                            active,
                            &video.errors,
                            &href,
                            &video_icon,
                            slug,
                            video.title.as_deref(),
                            translations,
                            video.unlisted
                        );

                        formatdoc!(r#"
                            <li>
                                {node}
                            </li>
                        "#)
                    })
                    .collect::<Vec<String>>()
                    .join("")
            };

            let node = tree_node(
                root_active,
                &playlist.errors,
                "/playlist/",
                &playlist_icon,
                site_dir,
                playlist.title.as_deref(),
                translations,
                playlist.unlisted
            );

            formatdoc!(r#"
                {node}
                <ul>
                    {videos}
                </ul>
            "#)
        }
        SiteContent::Video(video) => {
            let t_video = translations.video;
            let video_icon = icons::video(t_video);

            if path.is_some() {
                let breadcrumb_title = video.title.as_deref().unwrap_or(t_video);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                let (icon, space) = if root_active { (video_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/video/">{icon}{space}{e_breadcrumb_title}</a>"#));
            }

            tree_node(
                root_active,
                &video.errors,
                "/video/",
                &video_icon,
                site_dir,
                video.title.as_deref(),
                translations,
                video.unlisted
            )
        }
    };

    let breadcrumbs = if breadcrumbs.is_empty() {
        String::new()
    } else {
        let breadcrumbs_joined = breadcrumbs.join(" › ");
        formatdoc!(r#"
            <div class="breadcrumbs">
                {breadcrumbs_joined}
            </div>
        "#)
    };

    (breadcrumbs, site_content)
}

fn tree_node(
    active: bool,
    errors: &[String],
    href: &str,
    icon: &str,
    slug: &str,
    title_option: Option<&str>,
    translations: &Translations,
    unlisted: bool
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
    let e_title = title_option
        .map(|title| html_escape_outside_attribute(title))
        .unwrap_or(format!(r#"<span aria-hidden="true" class="no_title">…</span>"#));

    let title_with_unlisted_badge = if unlisted {
        let t_unlisted = translations.unlisted;
        format!(r#"<span class="unlisted_badge">{t_unlisted}</span> {e_title}"#)
    } else {
        e_title
    };

    let depth = if href.ends_with('/') { 0 } else { href.split('/').count() - 2 };
    let padding_left_rem = 0.4 + depth as f32 * 1.4;
    let padding_left = format!("{}rem", padding_left_rem);

    formatdoc!(r#"
        <a{active} href="{href}" style="padding-left: {padding_left};">
            <div class="icon">
                {icon_container}
            </div>
            <div class="labels">
                <div class="title">{title_with_unlisted_badge}</div>
                <div class="path">{slug}</div>
            </div>
        </a>
    "#)
}

pub fn upload_widget(action: &str, label: &str) -> String {
    formatdoc!(r#"
        <form action="{action}" method="post" enctype="multipart/form-data">
            <input name="file" type="file">
            <button class="action upload_button">{label}</button>
        </form>
    "#)
}
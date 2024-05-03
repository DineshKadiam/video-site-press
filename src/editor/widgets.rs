use std::cmp::Ordering;
use std::path::Path;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Field,
    Language,
    Playlist,
    Site,
    SiteContent,
    Translations,
    Video
};
use crate::icons;
use crate::util::human_size;

fn collection_tree(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    context: &Context,
    path: Option<&str>,
    translations: &Translations
) -> String {
    let collection_icon = icons::collection(translations.collection);
    let playlist_icon = icons::playlist(translations.playlist);
    let video_icon = icons::video(translations.video);
    let mut subcollections_sorted: Vec<Arc<Collection>> = collection.subcollections.to_vec();

    subcollections_sorted.sort_unstable_by(|a, b| {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    });

    let subcollections = subcollections_sorted
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
                let (icon, space) = if active { (collection_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="{href}">{icon}{space}{label}</a>"#));
            }

            let collection_content = collection_tree(breadcrumbs, collection, context, path, translations);

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
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    });

    let playlists = playlists_sorted
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
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    });

    let videos = videos_sorted
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
    let (breadcrumbs, tree) = site_tree(context, path, site, &context.site_dir, &language.translations);

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

    let language_options = [("de", "Deutsch"), ("en", "English")]
        .into_iter()
        .map(|(code, label)| {
            let selected = if code == &language.code { "selected" } else { "" };
            format!(r#"<option {selected} value="{code}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let external_icon = icons::external(language.translations.external_link);
    let t_close = language.translations.close;
    let t_language = language.translations.language;
    let splash = formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <img src="/splash.jpg?{hyper8_version}">
                <div class="pad">
                    <ul autofocus>
                        <li>Navigate the site's content in the left sidebar</li>
                        <li>Make changes in the main area of the page</li>
                        <li>Use the controls top-right to preview/deploy</li>
                        <li>Check the <a href="https://simonrepp.com/hyper8/docs" target="_blank">{external_icon} docs</a> for more information</li>
                    </ul>
                    <form action="/change-editor-language" method="post">
                        <input name="redirect" type="hidden">
                        <label for="change_editor_language">{t_language}</label>
                        <select id="change_editor_language" name="language">
                            {language_options}
                        </select>
                    </form>
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

    let t_deploy = language.translations.deploy;
    let deploy_icon = icons::deploy(t_deploy);
    let deploy_button = match site.deploy_config.complete() {
        true => {
            let t_really_deploy = language.translations.really_deploy;
            format!(r#"<form action="/deploy" method="post" onsubmit="return confirm('{t_really_deploy}');"><button>{deploy_icon} {t_deploy}</button></form>"#)
        }
        false => {
            let action = match site.content {
                SiteContent::Collection(_) => "/collection/",
                SiteContent::Playlist(_) => "/playlist/",
                SiteContent::Video(_) => "/video/",
                // TODO: Is there any point to implementing a site form for an uninitialized site?
                //       (at this point offering a link to configure deployment, when there is no
                //       way to configure anything, is anyhow definitely wrong).
                _ => "/"
            };

            // TODO: Hash that goes directly to the relevant section for editing site? (or dedicated page to clean up interface by area of concern?)
            let t_configure = language.translations.configure;
            let t_deploy_configuration_missing_required_fields = language.translations.deploy_configuration_missing_required_fields;
            format!(r#"<form action="{action}"><button title="{t_deploy_configuration_missing_required_fields}">{deploy_icon} {t_configure}</button></form>"#)
        }
    };

    let storage_usage = human_size(site.storage_usage());
    let logo_icon = icons::logo(language.translations.hyper8_logo);
    let t_idle = language.translations.idle;
    let t_preview = language.translations.preview;
    let t_processing = language.translations.processing;
    let preview_icon = icons::preview(t_preview);
    let processing_icon = icons::processing(t_processing);
    let storage_icon = icons::storage(language.translations.storage_usage);
    let t_about = language.translations.about;
    let t_filter = language.translations.filter;
    let body = formatdoc!(r#"
            <body data-version="{hyper8_version}">
                <div class="controls">
                    <form action="/jobs"><button class="processing">{processing_icon} {t_idle}</button></form>
                    <form action="/"><button>{storage_icon} {storage_usage}</button></form>
                    <form action="/build" method="post" target="_blank"><button>{preview_icon} {t_preview}</button></form>
                    {deploy_button}
                    <button id="toggle-splash">{logo_icon} {t_about}</button>
                </div>
                <div class="split">
                    <nav>
                        <input id="filter_tree" placeholder="{t_filter}" type="search">
                        {tree}
                    </nav>
                    <main>
                        {breadcrumbs}
                        {body}
                    </main>
                </div>
                {splash}
                <template id="processing_icon">
                    {processing_icon}
                </template>
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

pub fn not_found(
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
    let html = layout(context, language, None, site, &body, "Not Found");

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
                let (icon, space) = if root_active { (collection_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/collection/">{icon}{space}{breadcrumb_title}</a>"#));
            }

            let collection_content = collection_tree(&mut breadcrumbs, collection, context, path, translations);

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
                let (icon, space) = if root_active { (playlist_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/playlist/">{icon}{space}{breadcrumb_title}</a>"#));
            }

            let videos = if playlist.videos.is_empty() {
                formatdoc!(r#"
                    <li>
                        <span class="muted">Empty</span>
                    </li>
                "#)
            } else {
                let video_icon = icons::video(translations.video);

                let mut videos_sorted: Vec<Arc<Video>> = playlist.videos.to_vec();

                videos_sorted.sort_unstable_by(|a, b| {
                    match (&a.title, &b.title) {
                        (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
                        (None, None) => a.slug().cmp(&b.slug())
                    }
                });

                videos_sorted
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
                let (icon, space) = if root_active { (video_icon.as_str(), " ") } else { ("", "") };
                breadcrumbs.push(format!(r#"<a href="/video/">{icon}{space}{breadcrumb_title}</a>"#));
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
    let t_no_title = translations.no_title;
    let title = title_option
        .map(|title| title.to_string())
        .unwrap_or(format!(r#"<span class="muted">{t_no_title}</span>"#));

    let title_with_unlisted_badge = if unlisted {
        let t_unlisted = translations.unlisted;
        format!(r#"<div>{title} <span class="unlisted_badge">{t_unlisted}</span></div>"#)
    } else {
        title
    };

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
                <span class="title">{title_with_unlisted_badge}</span>
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
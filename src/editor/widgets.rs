use std::cmp::Ordering;
use std::path::Path;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Language,
    Link,
    Playlist,
    Site,
    SiteContent,
    Translations,
    Video
};
use crate::editor::Input;
use crate::icons;
use crate::util::{human_size, precise_duration};

/// Generic add link form, submit_url should be something like "/add-playlist-link/path/to/playlist"
pub fn add_link_form(submit_url: &str, translations: &Translations) -> String {
    let input_label = Input::text("label", translations.label_optional);
    let mut input_url = Input::url("url", translations.url);

    input_url.required();

    let t_add = translations.add;
    let t_hidden_link_with_rel_me_attribute = translations.hidden_link_with_rel_me_attribute;
    let t_just_a_regular_link_recommended_default = translations.just_a_regular_link_recommended_default;
    let t_link_type = translations.link_type;
    let t_link_with_rel_me_attribute = translations.link_with_rel_me_attribute;
    let t_new_link = translations.new_link;

    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_new_link}</h2>

            <form action="{submit_url}" method="post">
                {input_url}
                {input_label}
                <div class="form_field">
                    <label for="verification">{t_link_type}</label>
                    <select id="verification" name="verification">
                        <option selected>{t_just_a_regular_link_recommended_default}</option>
                        <option value="rel-me">{t_link_with_rel_me_attribute}</option>
                        <option value="rel-me-hidden">{t_hidden_link_with_rel_me_attribute}</option>
                    </select>
                </div>
                <button>{t_add}</button>
            </form>
        </div>
    "#)
}

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

pub fn form_textarea(
    key: &str,
    label: &str,
    value: Option<&str>
) -> String {
    let value = value.unwrap_or("");
    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <textarea id="{key}" name="{key}" rows="5">{value}</textarea>
        </div>
    "#)
}

pub fn form_textarea_markdown_subset(
    key: &str,
    label: &str,
    language: &Language,
    value: Option<&str>
) -> String {
    let hint = language.translations.markdown_inline_links_are_supported_standalone_urls_are_automatically_linked;
    let value = value.unwrap_or("");
    formatdoc!(r#"
        <div class="form_field">
            <label for="{key}">{label}</label>
            <div class="textarea_with_hint">
                <span>{hint}</span>
                <textarea id="{key}" name="{key}" rows="5">{value}</textarea>
            </div>
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

    let theme = context.get_editor_theme();

    let theme_options = [
        ("adaptive", language.translations.adaptive),
        ("dark", language.translations.dark),
        ("light", language.translations.light)
    ]
        .into_iter()
        .map(|(key, label)| {
            let selected = if &key == &theme.key { "selected" } else { "" };
            format!(r#"<option {selected} value="{key}">{label}</option>"#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let external_icon = icons::external(language.translations.external_link);
    let t_check_the_docs_for_more_information = language.translations.check_the_docs_for_more_information(&external_icon);
    let t_close = language.translations.close;
    let t_language = language.translations.language;
    let t_theme = language.translations.theme;
    let splash = formatdoc!(r#"
        <dialog class="splash_wrapper">
            <div class="splash">
                <img alt="Hyper 8 Video System" src="/splash_logo.svg?{hyper8_version}">
                <div class="pad">
                    <ul autofocus>
                        <!--li>Navigate the site's content in the left sidebar</li>
                        <li>Make changes in the main area of the page</li>
                        <li>Use the controls top-right to preview/deploy</li-->
                        <li>{t_check_the_docs_for_more_information}</li>
                    </ul>
                    <form action="/change-editor-config" method="post">
                        <input name="redirect" type="hidden">
                            <label for="language_code">{t_language}</label>
                            <select id="language_code" name="language_code">
                                {language_options}
                            </select>
                            <span style="display: inline-block; width: 1rem;"></span>
                            <label for="theme_key">{t_theme}</label>
                            <select id="theme_key" name="theme_key">
                                {theme_options}
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

    let t_configure_deployment = language.translations.configure_deployment;
    let configure_icon = icons::configure(language.translations.configure);
    let deployment_icon = icons::deployment(language.translations.deployment);
    let deployment_button = match site.deploy_config.complete() {
        true => {
            let deployment = &context.deploying.lock().unwrap();
            if deployment.active() {
                let t_deploying  = language.translations.deploying;
                let duration = precise_duration(deployment.started_at.unwrap().elapsed().map(|duration| duration.as_secs_f32()).unwrap_or(0.0));
                formatdoc!(r#"
                    <a href="/deployment">
                        {deployment_icon} {t_deploying} ({duration})
                    </a>
                "#)
            } else {
                let t_deploy = language.translations.deploy;
                let t_really_deploy = language.translations.really_deploy;
                formatdoc!(r#"
                    <form action="/deploy" method="post" onsubmit="return confirm('{t_really_deploy}');">
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

    let t_processing = language.translations.processing;
    let processing_icon = icons::processing(t_processing);
    let processing_link = {
        let processing = &context.processing.lock().unwrap();
        if processing.operations.is_empty() {
            let t_idle = language.translations.idle;
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

    let storage_usage = human_size(site.storage_usage());
    let logo_icon = icons::logo(language.translations.hyper8_logo);
    let t_preview = language.translations.preview;
    let preview_icon = icons::preview(t_preview);
    let storage_icon = icons::storage(language.translations.storage_usage);
    let t_about = language.translations.about;
    let t_filter = language.translations.filter;
    let body = formatdoc!(r#"
            <body data-version="{hyper8_version}">
                <div class="controls">
                    {processing_link}
                    <a href="/">{storage_icon} {storage_usage}</a>
                    <form action="/build" method="post" target="_blank"><button>{preview_icon} {t_preview}</button></form>
                    <div id="deployment">{deployment_button}</div>
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
                <template id="configure_icon">
                    {configure_icon}
                </template>
                <template id="deployment_icon">
                    {deployment_icon}
                </template>
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

/// Generic links listing with possibility to delete items, submit_url should be something like "/delete-playlist-link/path/to/playlist"
pub fn links(links: &[Link], submit_url: &str, translations: &Translations) -> String {
    if links.is_empty() {
        return String::new();
    }

    let t_delete = translations.delete;
    let t_hidden = translations.hidden;
    let t_really_delete = translations.really_delete;
    let delete_icon = icons::failure(t_delete);

    let links_rendered = links
        .iter()
        .map(|link| {
            let label = &link.label.as_deref().unwrap_or("");
            let url = &link.url;

            let delete_form = formatdoc!(r#"
                <form action="{submit_url}" method="post" onsubmit="return confirm('{t_really_delete}');" style="display: inline;">
                    <input name="label" type="hidden" value="{label}">
                    <input name="url" type="hidden" value="{url}">
                    <button class="delete_button">{delete_icon}</button>
                </form>
            "#);

            let hidden = if link.hidden { format!("[{t_hidden}]") } else { String::new() };
            let rel_me = if link.rel_me { r#"[rel="me"]"# } else { "" };

            formatdoc!(r#"
                <div class="button_group">
                    <span>{label} {rel_me} {hidden}<br>{url}</span>
                    {delete_form}
                </div>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let t_links = translations.links;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_links}</h2>
            {links_rendered}
        </div>
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
// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Site,
    SiteContent,
    Translations,
    TreeViewOrder
};
use crate::icons;
use crate::util::html_escape_outside_attribute;

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

pub fn tree_view(
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

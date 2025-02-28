// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    Playlist,
    Site,
    SiteContent,
    Translations,
    TreeViewOrder,
    Video
};
use crate::icons;
use crate::util::html_escape_outside_attribute;

struct Visibility {
    offline_inherited: bool,
    offline_local: bool,
    unlisted_inherited: bool,
    unlisted_local: bool
}

impl Visibility {
    pub fn from_collection(collection: &Collection) -> Visibility {
        Visibility {
            offline_inherited: collection.offline_inherited(),
            offline_local: collection.offline_local(),
            unlisted_inherited: collection.unlisted_inherited(),
            unlisted_local: collection.unlisted_local()
        }
    }

    pub fn from_playlist(playlist: &Playlist) -> Visibility {
        Visibility {
            offline_inherited: playlist.offline_inherited(),
            offline_local: playlist.offline_local(),
            unlisted_inherited: playlist.unlisted_inherited(),
            unlisted_local: playlist.unlisted_local()
        }
    }

    pub fn from_video(video: &Video) -> Visibility {
        Visibility {
            offline_inherited: video.offline_inherited(),
            offline_local: video.offline_local(),
            unlisted_inherited: video.unlisted_inherited(),
            unlisted_local: video.unlisted_local()
        }
    }

    pub fn none() -> Visibility {
        Visibility {
            offline_inherited: false,
            offline_local: false,
            unlisted_inherited: false,
            unlisted_local: false
        }
    }

    pub fn offline(&self) -> bool {
        self.offline_local || self.offline_inherited
    }

    pub fn offline_inherited_only(&self) -> bool {
        self.offline_inherited && !self.offline_local
    }

    pub fn unlisted(&self) -> bool {
        self.unlisted_local || self.unlisted_inherited
    }

    pub fn unlisted_inherited_only(&self) -> bool {
        self.unlisted_inherited && !self.unlisted_local
    }
}

fn collection_tree(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    path: Option<&str>,
    translations: &Translations,
    tree_view_order: &TreeViewOrder,
) -> String {
    let collection_icon = icons::collection(translations.collection);
    let playlist_icon = icons::playlist(translations.playlist);
    let video_icon = icons::video(translations.video);
    let subcollections = collection.subcollections_in_tree_view_order(tree_view_order)
        .iter()
        .map(|collection| {
            let collection_path = &collection.path;

            let collection_visibility = Visibility::from_collection(collection);

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
                &collection_visibility
            );

            if !active && path.is_some_and(|path|
                path
                    .strip_prefix(&collection.path)
                    .is_some_and(|rest| rest.starts_with('/'))
            ) {
                let label = collection.title.as_deref().unwrap_or_else(|| collection.slug());
                let e_label = html_escape_outside_attribute(label);
                breadcrumbs.push(format!(r#"<a href="{href}">{e_label}</a>"#));
            }

            let collection_content = collection_tree(
                breadcrumbs,
                collection,
                path,
                translations,
                tree_view_order,
            );

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

            let playlist_visibility = Visibility::from_playlist(playlist);

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
                &playlist_visibility
            );

            if !active && path.is_some_and(|path|
                path
                    .strip_prefix(&playlist.path)
                    .is_some_and(|rest| rest.starts_with('/'))
            ) {
                let label = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());
                let e_label = html_escape_outside_attribute(label);
                breadcrumbs.push(format!(r#"<a href="{href}">{e_label}</a>"#));
            }

            let children = if playlist.videos.is_empty() {
                String::new()
            } else {
                let lis = playlist.videos_in_tree_view_order(tree_view_order)
                    .iter()
                    .map(|video| {
                        let video_path = &video.path;

                        let video_visibility = Visibility::from_video(video);

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
                            &video_visibility
                        );

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

            let video_visibility = Visibility::from_video(video);

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
                &video_visibility
            );

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

    let root_active = path.is_some_and(|path| path.is_empty());
    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let t_collection = translations.collection;

            if path.is_some() && !root_active {
                let breadcrumb_title = collection.title.as_deref().unwrap_or(t_collection);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                breadcrumbs.push(format!(r#"<a href="/collection/">{e_breadcrumb_title}</a>"#));
            }

            let tree_view_order = context.get_editor_tree_view_order();

            let collection_content = collection_tree(
                &mut breadcrumbs,
                collection,
                path,
                translations,
                &tree_view_order,
            );

            let collection_icon = icons::collection(t_collection);
            let collection_visibility = Visibility::from_collection(collection);
            let errors = if collection.errors.is_empty() { &site.errors } else { &collection.errors };
            let node = tree_node(
                root_active,
                errors,
                "/collection/",
                &collection_icon,
                site_dir,
                collection.title.as_deref(),
                translations,
                &collection_visibility
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
                &Visibility::none()
            )
        }
        SiteContent::Playlist(playlist) => {
            let t_playlist = translations.playlist;

            if path.is_some() && !root_active {
                let breadcrumb_title = playlist.title.as_deref().unwrap_or(t_playlist);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                breadcrumbs.push(format!(r#"<a href="/playlist/">{e_breadcrumb_title}</a>"#));
            }

            let playlist_visibility = Visibility::from_playlist(playlist);

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

                        let video_visibility = Visibility::from_video(video);

                        let node = tree_node(
                            active,
                            &video.errors,
                            &href,
                            &video_icon,
                            slug,
                            video.title.as_deref(),
                            translations,
                            &video_visibility
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

            let playlist_icon = icons::playlist(t_playlist);
            let node = tree_node(
                root_active,
                &playlist.errors,
                "/playlist/",
                &playlist_icon,
                site_dir,
                playlist.title.as_deref(),
                translations,
                &playlist_visibility
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

            if path.is_some() {
                let breadcrumb_title = video.title.as_deref().unwrap_or(t_video);
                let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
                breadcrumbs.push(format!(r#"<a href="/video/">{e_breadcrumb_title}</a>"#));
            }

            let video_visibility = Visibility::from_video(video);

            let video_icon = icons::video(t_video);
            tree_node(
                root_active,
                &video.errors,
                "/video/",
                &video_icon,
                site_dir,
                video.title.as_deref(),
                translations,
                &video_visibility
            )
        }
    };

    let breadcrumbs = if breadcrumbs.is_empty() {
        String::new()
    } else {
        let breadcrumbs_joined = breadcrumbs.join(" › ");
        formatdoc!(r#"
            <div class="breadcrumbs">
                {breadcrumbs_joined} ›
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
    visibility: &Visibility
) -> String {
    let active = if active { r#"class="active""# } else { "" };
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
        .map(html_escape_outside_attribute)
        .unwrap_or(String::from(r#"<span aria-hidden="true" class="no_title">…</span>"#));

    let mut badges = Vec::new();

    if visibility.offline() {
        let inherited = if visibility.offline_inherited_only() { "inherited" } else { "" };
        let offline_icon = icons::offline(translations.offline);
        let badge = format!(r#"<span class="visibility {inherited}">{offline_icon}</span>"#);
        badges.push(badge);
    }

    if visibility.unlisted() {
        let inherited = if visibility.unlisted_inherited_only() { "inherited" } else { "" };
        let unlisted_icon = icons::unlisted(translations.unlisted);
        let badge = format!(r#"<span class="visibility {inherited}">{unlisted_icon}</span>"#);
        badges.push(badge);
    }

    let title_with_visibility_badges = if badges.is_empty() {
        e_title
    } else {
        format!("{} {e_title}", badges.join(" "))
    };

    let depth = if href.ends_with('/') { 0 } else { href.split('/').count() - 2 };
    let padding_left_rem = 0.4 + depth as f32 * 1.4;
    let padding_left = format!("{}rem", padding_left_rem);

    formatdoc!(r#"
        <a {active} href="{href}" style="padding-left: {padding_left};">
            <div class="icon">
                {icon_container}
            </div>
            <div class="labels">
                <div class="title">{title_with_visibility_badges}</div>
                <div class="path">{slug}</div>
            </div>
        </a>
    "#)
}

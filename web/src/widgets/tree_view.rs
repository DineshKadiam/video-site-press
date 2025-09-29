// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use indoc::formatdoc;

use hyper8_core::{
    Collection,
    Container,
    Playlist,
    Site,
    SiteContent,
    SitePath,
    Video
};
use hyper8_core::util::html_escape_outside_attribute;
use hyper8_translations::Translations;

use crate::{
    Location,
    SessionState,
    Subpage,
    TreeViewOrder
};
use crate::routes;

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

/// For the root node `site_dir` is displayed instead of the slug, and it must
/// be passed there too, otherwise this panics.
fn collection_node(
    breadcrumbs: &mut Vec<String>,
    collection: &Collection,
    location: &Location,
    site: &Site,
    site_dir: Option<&str>,
    translations: &Translations,
    tree_view_order: &TreeViewOrder
) -> String {
    let collection_manifest_url = routes::collection_manifest(collection);
    let collection_settings_url = routes::collection_edit(collection);
    let site_manifest_url = routes::SITE_MANIFEST;

    let exposed_edit_url = if collection.manifest_errors.is_empty() {
        &collection_settings_url
    } else {
        &collection_manifest_url
    };

    let has_errors =
        !collection.dir_errors.is_empty() ||
        !collection.manifest_errors.is_empty() ||
        (collection.site_path.is_root_path() && !site.manifest_errors.is_empty());

    if location.intersects(&collection.site_path) {
        let breadcrumb_title = collection.title_or_slug_or_generic_label(translations);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{exposed_edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let collection_icon = hyper8_icons::collection(translations.collection);
    let collection_visibility = Visibility::from_collection(collection);

    let node = tree_node(
        has_errors,
        exposed_edit_url,
        &collection_icon,
        site_dir.unwrap_or_else(|| collection.slug_unchecked()),
        collection.title.as_deref(),
        translations,
        &collection_visibility
    );

    let containers = tree_view_order.ordered_containers(&collection, &site.language)
        .iter()
        .map(|container| {
            match container {
                Container::Collection(collection) => {
                    let node = collection_node(
                        breadcrumbs,
                        collection,
                        location,
                        site,
                        None,
                        translations,
                        tree_view_order
                    );

                    format!("<li>{node}</li>")
                }
                Container::Playlist(playlist) => {
                    let node = playlist_node(
                        breadcrumbs,
                        location,
                        playlist,
                        site,
                        None,
                        translations,
                        tree_view_order
                    );

                    format!("<li>{node}</li>")
                }
            }
        })
        .collect::<Vec<String>>()
        .join("");

    let videos = tree_view_order.ordered_videos(&site.language, &collection.video_order, &collection.videos)
        .iter()
        .map(|video| {
            let node = video_node(
                breadcrumbs,
                location,
                site,
                None,
                translations,
                video
            );

            format!("<li>{node}</li>")
        })
        .collect::<Vec<String>>()
        .join("");

    let node_with_subpages = if let Some(subpage) = location.active_subpage(&collection.site_path) {
        let mut subpages = Vec::new();

        let depth = collection_settings_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        if collection.manifest_errors.is_empty() {
            let collection_settings_active = if *subpage == Subpage::Edit { "active" } else { "" };
            let collection_settings_icon = hyper8_icons::edit(None);
            let t_collection_settings = translations.collection_settings;
            let collection_settings = formatdoc!(r#"
                <a class="{collection_settings_active} subpage" href="{collection_settings_url}" style="padding-left: {padding_left};">
                    {collection_settings_icon} {t_collection_settings}
                </a>
            "#);
            subpages.push(collection_settings);
        }

        if collection.site_path.is_root_path() && site.manifest_errors.is_empty() {
            let logo_monochrome = hyper8_icons::logo_monochrome(None);
            let t_site_settings = translations.site_settings;
            let site_settings_url = routes::SITE_EDIT;
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo_monochrome} {t_site_settings}
                </a>
            "#);

            subpages.push(site_settings);
        };

        if collection.manifest_errors.is_empty() {
            let links_active = if *subpage == Subpage::Links { "active" } else { "" };
            let links_count = collection.links.len();
            let link_icon = hyper8_icons::link(None);
            let links_url = routes::collection_links(collection);
            let t_links = translations.links;
            let links = formatdoc!(r#"
                <a class="{links_active} subpage" href="{links_url}" style="padding-left: {padding_left};">
                    {link_icon} {t_links} ({links_count})
                </a>
            "#);
            subpages.push(links);

            let banner_active = if *subpage == Subpage::Poster { "active" } else { "" };
            let banner_icon = hyper8_icons::EMBED; // TODO: Custom banner/image icon
            let banner_url = routes::collection_banner(collection);
            let t_banner_image = translations.banner_image;
            let banner_image = formatdoc!(r#"
                <a class="{banner_active} subpage" href="{banner_url}" style="padding-left: {padding_left};">
                    {banner_icon} {t_banner_image}
                </a>
            "#);
            subpages.push(banner_image);

            let danger_icon = hyper8_icons::DANGER;
            let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
            let dangerous_actions_url = routes::collection_dangerous_actions(collection);
            let t_dangerous_actions = translations.dangerous_actions;
            let dangerous_actions = formatdoc!(r#"
                <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                    {danger_icon} {t_dangerous_actions}
                </a>
            "#);
            subpages.push(dangerous_actions);
        }

        let collection_manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };
        let collection_manifest_icon = if collection.manifest_errors.is_empty() {
            String::from(hyper8_icons::CODE)
        } else {
            icon_with_error(hyper8_icons::CODE)
        };
        let t_collection_manifest = translations.collection_manifest;
        let collection_manifest = formatdoc!(r#"
            <a class="{collection_manifest_active} subpage" href="{collection_manifest_url}" style="padding-left: {padding_left};">
                {collection_manifest_icon} {t_collection_manifest}
            </a>
        "#);
        subpages.push(collection_manifest);

        if collection.site_path.is_root_path() {
            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_manifest_icon = if site.manifest_errors.is_empty() {
                String::from(hyper8_icons::CODE)
            } else {
                icon_with_error(hyper8_icons::CODE)
            };
            let t_site_manifest = translations.site_manifest;

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {site_manifest_icon} {t_site_manifest}
                </a>
            "#);

            subpages.push(site_manifest);
        };

        let r_subpages = subpages.join("\n");

        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    {r_subpages}
                </div>
            </div>
        "#)
    } else {
        node
    };

    formatdoc!(r#"
        {node_with_subpages}
        <ul>
            {videos}
            {containers}
        </ul>
    "#)
}

fn icon_with_error(icon: &str) -> String {
    format!(r#"
        <span style="position: relative;">
            <span class="errors_badge"></span>
            {icon}
        </span>
    "#)
}

/// For the root node `site_dir` is displayed instead of the slug, and it must
/// be passed there too, otherwise this panics.
fn playlist_node(
    breadcrumbs: &mut Vec<String>,
    location: &Location,
    playlist: &Playlist,
    site: &Site,
    site_dir: Option<&str>,
    translations: &Translations,
    tree_view_order: &TreeViewOrder
) -> String {
    let playlist_manifest_url = routes::playlist_manifest(playlist);
    let playlist_settings_url = routes::playlist_edit(playlist);
    let site_manifest_url = routes::SITE_MANIFEST;

    let exposed_edit_url = if playlist.manifest_errors.is_empty() {
        &playlist_settings_url
    } else {
        &playlist_manifest_url
    };

    let has_errors =
        !playlist.dir_errors.is_empty() ||
        !playlist.manifest_errors.is_empty() ||
        playlist.site_path.is_root_path() && !site.manifest_errors.is_empty();

    if location.intersects(&playlist.site_path) {
        let breadcrumb_title = playlist.title_or_slug_or_generic_label(translations);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{exposed_edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let playlist_icon = hyper8_icons::playlist(Some(translations.playlist));
    let playlist_visibility = Visibility::from_playlist(playlist);

    let node = tree_node(
        has_errors,
        exposed_edit_url,
        &playlist_icon,
        site_dir.unwrap_or_else(|| playlist.slug_unchecked()),
        playlist.title.as_deref(),
        translations,
        &playlist_visibility
    );

    let node_with_subpages = if let Some(subpage) = location.active_subpage(&playlist.site_path) {
        let mut subpages = Vec::new();

        let depth = playlist_settings_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        if playlist.manifest_errors.is_empty() {
            let playlist_settings_active = if *subpage == Subpage::Edit { "active" } else { "" };
            let playlist_settings_icon = hyper8_icons::edit(None);
            let t_playlist_settings = translations.playlist_settings;
            let playlist_settings = formatdoc!(r#"
                <a class="{playlist_settings_active} subpage" href="{playlist_settings_url}" style="padding-left: {padding_left};">
                    {playlist_settings_icon} {t_playlist_settings}
                </a>
            "#);
            subpages.push(playlist_settings);
        }

        if playlist.site_path.is_root_path() && site.manifest_errors.is_empty() {
            let logo_monochrome = hyper8_icons::logo_monochrome(None);
            let t_site_settings = translations.site_settings;
            let site_settings_url = routes::SITE_EDIT;
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo_monochrome} {t_site_settings}
                </a>
            "#);

            subpages.push(site_settings);
        };

        if playlist.manifest_errors.is_empty() {
            let links_active = if *subpage == Subpage::Links { "active" } else { "" };
            let links_count = playlist.links.len();
            let link_icon = hyper8_icons::link(None);
            let links_url = routes::playlist_links(playlist);
            let t_links = translations.links;
            let links = formatdoc!(r#"
                <a class="{links_active} subpage" href="{links_url}" style="padding-left: {padding_left};">
                    {link_icon} {t_links} ({links_count})
                </a>
            "#);
            subpages.push(links);

            let banner_active = if *subpage == Subpage::Poster { "active" } else { "" };
            let banner_icon = hyper8_icons::EMBED; // TODO: Custom banner/image icon
            let banner_url = routes::playlist_banner(playlist);
            let t_banner_image = translations.banner_image;
            let banner_image = formatdoc!(r#"
                <a class="{banner_active} subpage" href="{banner_url}" style="padding-left: {padding_left};">
                    {banner_icon} {t_banner_image}
                </a>
            "#);
            subpages.push(banner_image);

            let danger_icon = hyper8_icons::DANGER;
            let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
            let dangerous_actions_url = routes::playlist_dangerous_actions(playlist);
            let t_dangerous_actions = translations.dangerous_actions;
            let dangerous_actions = formatdoc!(r#"
                <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                    {danger_icon} {t_dangerous_actions}
                </a>
            "#);
            subpages.push(dangerous_actions);
        }

        let playlist_manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };
        let playlist_manifest_icon = if playlist.manifest_errors.is_empty() {
            String::from(hyper8_icons::CODE)
        } else {
            icon_with_error(hyper8_icons::CODE)
        };
        let t_playlist_manifest = translations.playlist_manifest;
        let playlist_manifest = formatdoc!(r#"
            <a class="{playlist_manifest_active} subpage" href="{playlist_manifest_url}" style="padding-left: {padding_left};">
                {playlist_manifest_icon} {t_playlist_manifest}
            </a>
        "#);
        subpages.push(playlist_manifest);

        if playlist.site_path.is_root_path() {
            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_manifest_icon = if site.manifest_errors.is_empty() {
                String::from(hyper8_icons::CODE)
            } else {
                icon_with_error(hyper8_icons::CODE)
            };
            let t_site_manifest = translations.site_manifest;

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {site_manifest_icon} {t_site_manifest}
                </a>
            "#);

            subpages.push(site_manifest);
        };

        let r_subpages = subpages.join("\n");

        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    {r_subpages}
                </div>
            </div>
        "#)
    } else {
        node
    };

    if playlist.videos.is_empty() {
        node_with_subpages
    } else {
        let li_items = tree_view_order.ordered_videos(&site.language, &playlist.order, &playlist.videos)
            .iter()
            .map(|video| {
                let node = video_node(
                    breadcrumbs,
                    location,
                    site,
                    None,
                    translations,
                    video
                );

                format!("<li>{node}</li>")
            })
            .collect::<Vec<String>>()
            .join("");

        formatdoc!("
            {node_with_subpages}
            <ul>{li_items}</ul>
        ")
    }
}

pub fn tree_view(
    location: Location,
    session_state: &SessionState,
    site: &Site,
    site_dir: &Path
) -> (String, String) {
    let tree_view_order = session_state.tree_view_order;
    let translations = &session_state.language.translations;

    let mut breadcrumbs = Vec::new();

    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            collection_node(
                &mut breadcrumbs,
                collection,
                &location,
                site,
                Some(site_dir),
                translations,
                &tree_view_order
            )
        }
        SiteContent::Empty => {
            let dashboard_url = routes::DASHBOARD;

            let t_empty_site = translations.empty_site;
            let collection_icon = hyper8_icons::disk(t_empty_site);

            let active = location.active_subpage(&SitePath::root_path()).is_some();

            if active {
                breadcrumbs.push(format!(r#"<a href="{dashboard_url}">{root_dir}</a>"#));
            }

            tree_node(
                false,
                dashboard_url,
                &collection_icon,
                site_dir,
                Some(t_empty_site),
                translations,
                &Visibility::none()
            )
        }
        SiteContent::Invalid { .. } => {
            let dashboard_url = routes::DASHBOARD;

            let t_invalid_site = translations.invalid_site;
            let collection_icon = hyper8_icons::configure(Some(t_invalid_site));

            let active = location.active_subpage(&SitePath::root_path()).is_some();

            if active {
                breadcrumbs.push(format!(r#"<a href="{dashboard_url}">{root_dir}</a>"#));
            }

            tree_node(
                true,
                dashboard_url,
                &collection_icon,
                site_dir,
                Some(t_invalid_site),
                translations,
                &Visibility::none()
            )
        }
        SiteContent::Playlist(playlist) => {
            playlist_node(
                &mut breadcrumbs,
                &location,
                playlist,
                site,
                Some(site_dir),
                translations,
                &tree_view_order
            )
        }
        SiteContent::Video(video) => {
            video_node(
                &mut breadcrumbs,
                &location,
                site,
                Some(site_dir),
                translations,
                video
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
    has_errors: bool,
    href: &str,
    icon: &str,
    slug_or_path: &str,
    title_option: Option<&str>,
    translations: &Translations,
    visibility: &Visibility
) -> String {
    let icon_container = if has_errors {
        icon_with_error(icon)
    } else {
        icon.to_string()
    };

    let e_title = title_option
        .map(html_escape_outside_attribute)
        .unwrap_or_else(|| {
            let t_no_title = translations.no_title;
            format!(r#"<span aria-hidden="true" class="no_title">{t_no_title}</span>"#)
        });

    let mut badges = Vec::new();

    if visibility.offline() {
        let inherited = if visibility.offline_inherited_only() { "inherited" } else { "" };
        let offline_icon = hyper8_icons::offline(translations.offline);
        let badge = format!(r#"<span class="visibility {inherited}">{offline_icon}</span>"#);
        badges.push(badge);
    }

    if visibility.unlisted() {
        let inherited = if visibility.unlisted_inherited_only() { "inherited" } else { "" };
        let unlisted_icon = hyper8_icons::unlisted(translations.unlisted);
        let badge = format!(r#"<span class="visibility {inherited}">{unlisted_icon}</span>"#);
        badges.push(badge);
    }

    let title_with_visibility_badges = if badges.is_empty() {
        e_title
    } else {
        format!("{} {e_title}", badges.join(" "))
    };

    let depth = std::cmp::max(href.split('/').count() as i32, 4) - 4;
    let padding_left_rem = 0.6 + depth as f32 * 1.4;
    let padding_left = format!("{}rem", padding_left_rem);

    formatdoc!(r#"
        <a href="{href}" style="padding-left: {padding_left};">
            <div class="icon">
                {icon_container}
            </div>
            <div class="labels">
                <div class="title">{title_with_visibility_badges}</div>
                <div class="path">{slug_or_path}</div>
            </div>
        </a>
    "#)
}

/// For the root node `site_dir` is displayed instead of the slug, and it must
/// be passed there too, otherwise this panics.
fn video_node(
    breadcrumbs: &mut Vec<String>,
    location: &Location,
    site: &Site,
    site_dir: Option<&str>,
    translations: &Translations,
    video: &Video
) -> String {
    let site_manifest_url = routes::SITE_MANIFEST;
    let video_manifest_url = routes::video_manifest(video);
    let video_settings_url = routes::video_edit(video);

    let exposed_edit_url = if video.manifest_errors.is_empty() {
        &video_settings_url
    } else {
        &video_manifest_url
    };

    let has_errors =
        !video.dir_errors.is_empty() ||
        !video.manifest_errors.is_empty() ||
        video.site_path.is_root_path() && !site.manifest_errors.is_empty();

    if location.intersects(&video.site_path) {
        let breadcrumb_title = video.title.as_deref().unwrap_or(translations.video);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{exposed_edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let video_icon = hyper8_icons::video(translations.video);
    let visibility = Visibility::from_video(video);

    let node = tree_node(
        has_errors,
        exposed_edit_url,
        &video_icon,
        site_dir.unwrap_or_else(|| video.slug_unchecked()),
        video.title.as_deref(),
        translations,
        &visibility
    );

    if let Some(subpage) = location.active_subpage(&video.site_path) {
        let mut subpages = Vec::new();

        let depth = video_settings_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        if video.manifest_errors.is_empty() {
            let video_settings_active = if *subpage == Subpage::Edit { "active" } else { "" };
            let video_settings_icon = hyper8_icons::edit(None);
            let t_video_settings = translations.video_settings;
            let video_settings = formatdoc!(r#"
                <a class="{video_settings_active} subpage" href="{video_settings_url}" style="padding-left: {padding_left};">
                    {video_settings_icon} {t_video_settings}
                </a>
            "#);
            subpages.push(video_settings);
        }

        if video.site_path.is_root_path() && site.manifest_errors.is_empty() {
            let logo_monochrome = hyper8_icons::logo_monochrome(None);
            let t_site_settings = translations.site_settings;
            let site_settings_url = routes::SITE_EDIT;
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo_monochrome} {t_site_settings}
                </a>
            "#);

            subpages.push(site_settings);
        };

        if video.manifest_errors.is_empty() {
            let media_active = if *subpage == Subpage::Media { "active" } else { "" };
            let media_url = routes::video_media(video);
            let storage_icon = hyper8_icons::storage(None);
            let video_audio_files_count = video.video_files.len() + video.audio_files.len();
            let t_video_audio_files = translations.video_audio_files;
            let video_audio_files = formatdoc!(r#"
                <a class="{media_active} subpage" href="{media_url}" style="padding-left: {padding_left};">
                    {storage_icon} {t_video_audio_files} ({video_audio_files_count})
                </a>
            "#);
            subpages.push(video_audio_files);

            let subtitles_active = if *subpage == Subpage::Subtitles { "active" } else { "" };
            let subtitles_count = video.subtitle_files.len();
            let subtitles_icon = hyper8_icons::subtitles(None);
            let subtitles_url = routes::video_subtitles(video);
             let t_subtitles = translations.subtitles;
            let subtitles = formatdoc!(r#"
                <a class="{subtitles_active} subpage" href="{subtitles_url}" style="padding-left: {padding_left};">
                    {subtitles_icon} {t_subtitles} ({subtitles_count})
                </a>
            "#);
            subpages.push(subtitles);

            let links_active = if *subpage == Subpage::Links { "active" } else { "" };
            let links_count = video.links.len();
            let link_icon = hyper8_icons::link(None);
            let links_url = routes::video_links(video);
            let t_links = translations.links;
            let links = formatdoc!(r#"
                <a class="{links_active} subpage" href="{links_url}" style="padding-left: {padding_left};">
                    {link_icon} {t_links} ({links_count})
                </a>
            "#);
            subpages.push(links);

            // TODO: Indicate missing image description for poster image on the subpage link itself
            let poster_active = if *subpage == Subpage::Poster { "active" } else { "" };
            let poster_icon = hyper8_icons::EMBED; // TODO: Custom poster icon
            let poster_url = routes::video_poster(video);
            let t_poster_image = translations.poster_image;
            let poster_image = formatdoc!(r#"
                <a class="{poster_active} subpage" href="{poster_url}" style="padding-left: {padding_left};">
                    {poster_icon} {t_poster_image}
                </a>
            "#);
            subpages.push(poster_image);

            let danger_icon = hyper8_icons::DANGER;
            let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
            let dangerous_actions_url = routes::video_dangerous_actions(video);
            let t_dangerous_actions = translations.dangerous_actions;
            let dangerous_actions = formatdoc!(r#"
                <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                    {danger_icon} {t_dangerous_actions}
                </a>
            "#);
            subpages.push(dangerous_actions);
        }

        let video_manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };
        let video_manifest_icon = if video.manifest_errors.is_empty() {
            String::from(hyper8_icons::CODE)
        } else {
            icon_with_error(hyper8_icons::CODE)
        };
        let t_video_manifest = translations.video_manifest;
        let video_manifest = formatdoc!(r#"
            <a class="{video_manifest_active} subpage" href="{video_manifest_url}" style="padding-left: {padding_left};">
                {video_manifest_icon} {t_video_manifest}
            </a>
        "#);
        subpages.push(video_manifest);

        if video.site_path.is_root_path() {
            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_manifest_icon = if site.manifest_errors.is_empty() {
                String::from(hyper8_icons::CODE)
            } else {
                icon_with_error(hyper8_icons::CODE)
            };
            let t_site_manifest = translations.site_manifest;

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {site_manifest_icon} {t_site_manifest}
                </a>
            "#);

            subpages.push(site_manifest);
        };

        let r_subpages = subpages.join("\n");

        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    {r_subpages}
                </div>
            </div>
        "#)
    } else {
        node
    }
}

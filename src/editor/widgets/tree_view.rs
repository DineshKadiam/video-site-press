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
    SitePath,
    Translations,
    TreeViewOrder,
    Video
};
use crate::editor::{Location, Subpage};
use crate::editor::routes;
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
    let edit_url = routes::collection_edit(collection);

    if location.intersects(&collection.site_path) {
        let breadcrumb_title = collection.title_or_slug_or_generic_label(translations);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let active = location.active_subpage(&collection.site_path).is_some();
    let collection_icon = icons::collection(translations.collection);
    let collection_visibility = Visibility::from_collection(collection);

    let has_errors = !collection.errors.is_empty() ||
        (collection.site_path.is_root_path() && !site.errors.is_empty());

    let node = tree_node(
        active,
        has_errors,
        &edit_url,
        &collection_icon,
        site_dir.unwrap_or_else(|| collection.slug_unchecked()),
        collection.title.as_deref(),
        translations,
        &collection_visibility
    );

    let subcollections = collection.subcollections_in_tree_view_order(tree_view_order)
        .iter()
        .map(|collection| {
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
        })
        .collect::<Vec<String>>()
        .join("");

    let playlists = collection.playlists_in_tree_view_order(tree_view_order)
        .iter()
        .map(|playlist| {
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
        })
        .collect::<Vec<String>>()
        .join("");

    let videos = collection.videos_in_tree_view_order(tree_view_order)
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
        let depth = edit_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        let dangerous_actions_url = routes::collection_dangerous_actions(collection);
        let edit_manifest_url = routes::collection_manifest(collection);

        let t_dangerous_actions = translations.dangerous_actions;
        let t_collection_manifest = translations.collection_manifest;
        let t_collection_settings = translations.collection_settings;

        let code_icon = icons::CODE;
        let danger_icon = icons::DANGER;
        let edit_icon = icons::EDIT;

        let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
        let edit_active = if *subpage == Subpage::Edit { "active" } else { "" };
        let manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };

        let (site_manifest, site_settings) = if collection.site_path.is_root_path() {
            let logo = icons::logo(None);

            let t_site_manifest = translations.site_manifest;
            let t_site_settings = translations.site_settings;

            let site_manifest_url = routes::SITE_MANIFEST;
            let site_settings_url = routes::SITE_EDIT;

            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {code_icon} {t_site_manifest}
                </a>
            "#);

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo} {t_site_settings}
                </a>
            "#);

            (site_manifest, site_settings)
        } else {
            (String::new(), String::new())
        };

        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    <a class="{edit_active} subpage" href="{edit_url}" style="padding-left: {padding_left};">
                        {edit_icon} {t_collection_settings}
                    </a>
                    {site_settings}
                    <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                        {danger_icon} {t_dangerous_actions}
                    </a>
                    <a class="{manifest_active} subpage" href="{edit_manifest_url}" style="padding-left: {padding_left};">
                        {code_icon} {t_collection_manifest}
                    </a>
                    {site_manifest}
                </div>
            </div>
        "#)
    } else {
        node
    };

    formatdoc!(r#"
        {node_with_subpages}
        <ul>
            {subcollections}
            {playlists}
            {videos}
        </ul>
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
    let edit_url = routes::playlist_edit(playlist);

    if location.intersects(&playlist.site_path) {
        let breadcrumb_title = playlist.title_or_slug_or_generic_label(translations);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let playlist_icon = icons::playlist(translations.playlist);
    let playlist_visibility = Visibility::from_playlist(playlist);

    let has_errors = !playlist.errors.is_empty() ||
        (playlist.site_path.is_root_path() && !site.errors.is_empty());

    let node = tree_node(
        false,
        has_errors,
        &edit_url,
        &playlist_icon,
        site_dir.unwrap_or_else(|| playlist.slug_unchecked()),
        playlist.title.as_deref(),
        translations,
        &playlist_visibility
    );

    let node_with_subpages = if let Some(subpage) = location.active_subpage(&playlist.site_path) {
        let depth = edit_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        let dangerous_actions_url = routes::playlist_dangerous_actions(playlist);
        let edit_manifest_url = routes::playlist_manifest(playlist);

        let t_dangerous_actions = translations.dangerous_actions;
        let t_playlist_manifest = translations.playlist_manifest;
        let t_playlist_settings = translations.playlist_settings;

        let code_icon = icons::CODE;
        let danger_icon = icons::DANGER;
        let edit_icon = icons::EDIT;

        let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
        let edit_active = if *subpage == Subpage::Edit { "active" } else { "" };
        let manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };

        let (site_manifest, site_settings) = if playlist.site_path.is_root_path() {
            let logo = icons::logo(None);

            let t_site_manifest = translations.site_manifest;
            let t_site_settings = translations.site_settings;

            let site_manifest_url = routes::SITE_MANIFEST;
            let site_settings_url = routes::SITE_EDIT;

            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {code_icon} {t_site_manifest}
                </a>
            "#);

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo} {t_site_settings}
                </a>
            "#);

            (site_manifest, site_settings)
        } else {
            (String::new(), String::new())
        };

        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    <a class="{edit_active} subpage" href="{edit_url}" style="padding-left: {padding_left};">
                        {edit_icon} {t_playlist_settings}
                    </a>
                    {site_settings}
                    <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                        {danger_icon} {t_dangerous_actions}
                    </a>
                    <a class="{manifest_active} subpage" href="{edit_manifest_url}" style="padding-left: {padding_left};">
                        {code_icon} {t_playlist_manifest}
                    </a>
                    {site_manifest}
                </div>
            </div>
        "#)
    } else {
        node
    };

    if playlist.videos.is_empty() {
        node_with_subpages
    } else {
        let li_items = playlist.videos_in_tree_view_order(tree_view_order)
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
    context: &Context,
    location: Location,
    site: &Site,
    site_dir: &Path,
    translations: &Translations
) -> (String, String) {
    let mut breadcrumbs = Vec::new();

    let root_dir = site_dir.file_name().unwrap().to_string_lossy();
    let site_dir = &site_dir.to_string_lossy();

    let site_content = match &site.content {
        SiteContent::Collection(collection) => {
            let tree_view_order = context.get_editor_tree_view_order();
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
            let collection_icon = icons::disk(t_empty_site);

            let active = location.active_subpage(&SitePath::root_path()).is_some();

            if active {
                breadcrumbs.push(format!(r#"<a href="{dashboard_url}">{root_dir}</a>"#));
            }

            tree_node(
                active,
                false,
                dashboard_url,
                &collection_icon,
                site_dir,
                Some(t_empty_site),
                translations,
                &Visibility::none()
            )
        }
        SiteContent::Invalid => {
            let dashboard_url = routes::DASHBOARD;

            let t_invalid_site = translations.invalid_site;
            let collection_icon = icons::configure(Some(t_invalid_site));

            let active = location.active_subpage(&SitePath::root_path()).is_some();

            if active {
                breadcrumbs.push(format!(r#"<a href="{dashboard_url}">{root_dir}</a>"#));
            }

            tree_node(
                active,
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
            let tree_view_order = context.get_editor_tree_view_order();
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
    active: bool,
    has_errors: bool,
    href: &str,
    icon: &str,
    slug_or_path: &str,
    title_option: Option<&str>,
    translations: &Translations,
    visibility: &Visibility
) -> String {
    let active = if active { r#"class="active""# } else { "" };
    let icon_container = if has_errors {
        format!(r#"
            <span style="position: relative;">
                <span class="errors_badge"></span>
                {icon}
            </span>
        "#)
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

    let depth = std::cmp::max(href.split('/').count() as i32, 4) - 4;
    let padding_left_rem = 0.6 + depth as f32 * 1.4;
    let padding_left = format!("{}rem", padding_left_rem);

    formatdoc!(r#"
        <a {active} href="{href}" style="padding-left: {padding_left};">
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
    let edit_url = routes::video_edit(video);

    if location.intersects(&video.site_path) {
        let breadcrumb_title = video.title.as_deref().unwrap_or(translations.video);
        let e_breadcrumb_title = html_escape_outside_attribute(breadcrumb_title);
        breadcrumbs.push(format!(r#"<a href="{edit_url}">{e_breadcrumb_title}</a>"#));
    }

    let video_icon = icons::video(translations.video);
    let visibility = Visibility::from_video(video);

    let has_errors = !video.errors.is_empty() ||
        (video.site_path.is_root_path() && !site.errors.is_empty());

    let node = tree_node(
        false,
        has_errors,
        &edit_url,
        &video_icon,
        site_dir.unwrap_or_else(|| video.slug_unchecked()),
        video.title.as_deref(),
        translations,
        &visibility
    );

    if let Some(subpage) = location.active_subpage(&video.site_path) {
        let depth = edit_url.split('/').count() as f32 - 3.5;
        let padding_left_rem = 0.6 + depth * 1.4;
        let padding_left = format!("{}rem", padding_left_rem);

        let dangerous_actions_url = routes::video_dangerous_actions(video);
        let edit_manifest_url = routes::video_manifest(video);
        let media_url = routes::video_media(video);
        let poster_url = routes::video_poster(video);
        let subtitles_url = routes::video_subtitles(video);

        let t_dangerous_actions = translations.dangerous_actions;
        let t_video_files = translations.video_files;
        let t_poster_image = translations.poster_image;
        let t_subtitles = translations.subtitles;
        let t_video_manifest = translations.video_manifest;
        let t_video_settings = translations.video_settings;

        let code_icon = icons::CODE;
        let danger_icon = icons::DANGER;
        let edit_icon = icons::EDIT;
        let media_icon = icons::storage(None);
        let poster_icon = icons::EMBED; // TODO: Custom poster icon
        let subtitles_icon = icons::subtitles(None);

        let dangerous_actions_active = if *subpage == Subpage::DangerousActions { "active" } else { "" };
        let edit_active = if *subpage == Subpage::Edit { "active" } else { "" };
        let manifest_active = if *subpage == Subpage::Manifest { "active" } else { "" };
        let media_active = if *subpage == Subpage::Media { "active" } else { "" };
        let poster_active = if *subpage == Subpage::Poster { "active" } else { "" };
        let subtitles_active = if *subpage == Subpage::Subtitles { "active" } else { "" };

        let (site_manifest, site_settings) = if video.site_path.is_root_path() {
            let logo = icons::logo(None);

            let t_site_manifest = translations.site_manifest;
            let t_site_settings = translations.site_settings;

            let site_manifest_url = routes::SITE_MANIFEST;
            let site_settings_url = routes::SITE_EDIT;

            let site_manifest_active = if *subpage == Subpage::SiteManifest { "active" } else { "" };
            let site_settings_active = if *subpage == Subpage::SiteSettings { "active" } else { "" };

            let site_manifest = formatdoc!(r#"
                <a class="{site_manifest_active} subpage" href="{site_manifest_url}" style="padding-left: {padding_left};">
                    {code_icon} {t_site_manifest}
                </a>
            "#);

            let site_settings = formatdoc!(r#"
                <a class="{site_settings_active} subpage" href="{site_settings_url}" style="padding-left: {padding_left};">
                    {logo} {t_site_settings}
                </a>
            "#);

            (site_manifest, site_settings)
        } else {
            (String::new(), String::new())
        };

        let subtitles_count = video.subtitles.len();

        // TODO: Indicate missing image description for poster image on the poster image subpage link entry
        formatdoc!(r#"
            <div class="active">
                {node}
                <div class="subpages">
                    <a class="{edit_active} subpage" href="{edit_url}" style="padding-left: {padding_left};">
                        {edit_icon} {t_video_settings}
                    </a>
                    <a class="{poster_active} subpage" href="{poster_url}" style="padding-left: {padding_left};">
                        {poster_icon} {t_poster_image}
                    </a>
                    {site_settings}
                    <a class="{media_active} subpage" href="{media_url}" style="padding-left: {padding_left};">
                        {media_icon} {t_video_files}
                    </a>
                    <a class="{subtitles_active} subpage" href="{subtitles_url}" style="padding-left: {padding_left};">
                        {subtitles_icon} {t_subtitles} ({subtitles_count})
                    </a>
                    <a class="{dangerous_actions_active} subpage" href="{dangerous_actions_url}" style="padding-left: {padding_left};">
                        {danger_icon} {t_dangerous_actions}
                    </a>
                    <a class="{manifest_active} subpage" href="{edit_manifest_url}" style="padding-left: {padding_left};">
                        {code_icon} {t_video_manifest}
                    </a>
                    {site_manifest}
                </div>
            </div>
        "#)
    } else {
        node
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{
    Context,
    MarkdownSubset,
    PlatformIntegrationMeta,
    Playlist,
    Site,
    Video
};
use crate::build::{AssetHashes, Layout};
use crate::build::outline::{Traversal, TraversalStep};
use crate::build::player::player;
use crate::build::widgets::{copy_button, links, unlisted_badge};
use crate::icons;
use crate::util::{
    html_escape_outside_attribute,
    localized_date_d_mmmm_yyyy
};

/// Renders a video page. If the video is part of a playlist, the layout is
/// split in two columns (respectively rows, on mobile), to additionally
/// display the playlist sidebar.
pub fn video_html(
    asset_hashes: &AssetHashes,
    context: &Context,
    playlist: Option<&Playlist>,
    site: &Site,
    traversal: &Traversal,
    video: &Video
) {
    let mut layout = Layout::new();
    let translations = &site.language.translations;

    layout.add_video_script();

    let index_suffix = site.index_suffix();

    let description = if let Some(description) = &video.description {
        let description_html = MarkdownSubset::to_html(description);
        format!(r#"<div class="description reading_width">{description_html}</div>"#)
    } else {
        String::new()
    };

    let title = video.title_or_slug_or_generic_label(translations);
    let e_title = html_escape_outside_attribute(title);

    let parent_context = if traversal.steps.len() > 1 {
        let parent_step = &traversal.steps[traversal.steps.len() - 2];
        parent_step.context_with_bubbles("../", site)
    } else {
        String::new()
    };

    let video_prefix = "";
    let player_rendered = player(
        false,
        playlist,
        site,
        video,
        video_prefix
    );

    let mut actions = Vec::new();

    if video.copy_link() {
        let (content_key, content_value) = match &site.base_url {
            Some(base_url) => ("content", base_url.join_index(site, &video.site_path)),
            None => ("dynamic-url", String::new())
        };

        layout.add_clipboard_script();

        let copy_link = copy_button(content_key, &content_value, translations.copy_link);

        actions.push(copy_link);
    }

    if video.embedding() && site.base_url.is_some() && !video.video_files.is_empty() {
        let embed_icon = icons::EMBED;

        let t_embed = &translations.embed;
        let t_embed_codes_permalink = &translations.embed_codes_permalink;

        let embed_action = format!(r#"
            <a href="{t_embed_codes_permalink}{index_suffix}">
                {embed_icon} {t_embed}
            </a>
        "#);

        actions.push(embed_action);
    }

    if video.download() && !video.video_files.is_empty() {
        let t_download = translations.download;
        let t_download_permalink = translations.download_permalink;
        let download_icon = icons::DOWNLOAD;

        let download = formatdoc!(r#"
            <a href="{t_download_permalink}{index_suffix}">
                {download_icon} {t_download}
            </a>
        "#);

        actions.push(download);
    }

    if let Some(playlist) = playlist {
        let t_playlist = translations.playlist;
        let playlist_icon = icons::playlist(None);
        let playlist_site_path = playlist.site_path.normalized();
        let toggle_playlist = formatdoc!(r#"
            <button data-site-path="{playlist_site_path}" id="toggle_playlist">
                {playlist_icon} {t_playlist}
            </button>
        "#);

        actions.push(toggle_playlist);
    }

    let release_date = match video.release_date {
        Some(release_date) => {
            let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
            format!(r#"<div class="release_date">{formatted_date}</div>"#)
        }
        None => String::new()
    };

    let r_unlisted_badge = if video.unlisted() {
        layout.no_indexing();
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let links_rendered = links(&site.language, &video.links);

    // TODO: Expose/unhide synopsis if and when it makes sense again
    // let synopsis = if let Some(synopsis) = &video.synopsis {
    //     let synopsis_escaped = html_escape_outside_attribute(synopsis);
    //     format!("<p>{synopsis_escaped}</p>")
    // } else {
    //     String::new()
    // };

    let actions = if actions.is_empty() {
        String::new()
    } else {
        let actions_joined = actions.join("\n");
        formatdoc!(r#"
            <div class="actions align flex space">
                {actions_joined}
            </div>
        "#)
    };

    let video_content = formatdoc!(r#"
        <div>
            <div class="video_info top">
                <h1 class="video_title">{e_title} {r_unlisted_badge}</h1>
                {parent_context}
            </div>
            {player_rendered}
            <div class="video_info">
                <div class="align_start apart flex wrap" style="margin-top: 2rem;">
                    <div>
                        {release_date}
                    </div>
                    {actions}
                </div>
                {links_rendered}
                {description}
            </div>
        </div>
    "#);

    let body = match playlist {
        Some(playlist) => {
            let prefix = "../";
            let playlist_context = TraversalStep::crumb_playlist(
                playlist,
                &prefix,
                site,
                Some(video)
            );

            let playlist_site_path = playlist.site_path.normalized();
            formatdoc!(r#"
                <div class="layout_split">
                    {video_content}
                    <div class="sidebar">
                        {playlist_context}
                    </div>
                    <script>
                        if (sessionStorage.getItem('hyper8HidePlaylist:{playlist_site_path}') !== null) {{
                            const sidebar = document.currentScript.previousElementSibling;
                            sidebar.classList.add('hidden');
                        }}
                    </script>
                </div>
            "#)
        }
        None => video_content
    };

    let platform_integration = video.platform_integration();
    if platform_integration.enabled() {
        if let Some(video_url) = site.absolute_url_index(&video.site_path) {
            let mut meta = PlatformIntegrationMeta::new(title.to_string(), video_url);

            if let Some(description) = &video.description {
                let description_plaintext = MarkdownSubset::to_plaintext(description);

                // As a somewhat arbitrary practical decision we currently
                // only use "our" description as the description for platform integration
                // when it is less than 256 characters long in plaintext.
                if description_plaintext.len() < 256 {
                    meta.description(&description_plaintext);
                }
            }

            if let Some(poster) = &video.poster {
                if let Some(poster_assets) = &poster.assets {
                    let image_url_prefix = site.absolute_url_prefix_unchecked(&video.site_path);
                    let platform_integration_image = poster_assets.platform_integration_image("poster", &image_url_prefix);

                    meta.image(platform_integration_image);

                    if let Some(description) = &poster.description {
                        meta.image_alt(description);
                    }
                }
            }

            if platform_integration.content_embeds() {
                if let Some(video_file) = video.recommended_video_file() {
                    if let Some(video_meta) = &video_file.video_meta {
                        let player_url = site.absolute_url_dir_unchecked("embed/", &video.site_path);
                        let video_url = site.absolute_url_file_unchecked(&video_file.file_name, &video.site_path);

                        let height = video_meta.height;
                        let mime_type = video_file.mime_type();
                        let width = video_meta.width;

                        meta.embed(
                            height,
                            mime_type,
                            player_url,
                            video_url,
                            width
                        );
                    }
                }
            }

            layout.add_platform_integration_meta(meta);
        }
    }

    let html = layout.render(
        asset_hashes,
        &body,
        site,
        title,
        traversal
    );

    let video_dir = context.build_dir.join(video.site_path.filesystem_relative());
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::ptr;

use indoc::formatdoc;

use hyper8_util::{html_escape_outside_attribute, precise_duration};

use crate::{
    Build,
    Context,
    MarkdownSubset,
    PlatformIntegrationMeta,
    Playlist,
    Site,
    Subtitles,
    Video
};
use crate::build::Layout;
use crate::build::outline::{Traversal, TraversalStep};
use crate::build::player::player;
use crate::build::widgets::{copy_button, links, unlisted_badge};
use crate::util::localized_date_d_mmmm_yyyy;

/// Renders a video page. If the video is part of a playlist, the layout is
/// split in two columns (respectively rows, on mobile), to additionally
/// display the playlist sidebar.
pub fn video_html(
    build: &Build,
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

    if video.embedding() && site.base_url.is_some() && video.has_streamable_media() {
        let embed_icon = hyper8_icons::EMBED;

        let t_embed = &translations.embed;
        let t_embed_codes_permalink = &translations.embed_codes_permalink;

        let embed_action = format!(r#"
            <a href="{t_embed_codes_permalink}{index_suffix}">
                {embed_icon} {t_embed}
            </a>
        "#);

        actions.push(embed_action);
    }

    if video.download() && video.has_downloadable_media() {
        let t_download = translations.download;
        let t_download_permalink = translations.download_permalink;
        let download_icon = hyper8_icons::DOWNLOAD;

        let download = formatdoc!(r#"
            <a href="{t_download_permalink}{index_suffix}">
                {download_icon} {t_download}
            </a>
        "#);

        actions.push(download);
    }

    if let Some(playlist) = playlist {
        let t_playlist = translations.playlist;
        let playlist_icon = hyper8_icons::playlist(None);
        let playlist_site_path = playlist.site_path.normalized();
        let toggle_playlist = formatdoc!(r#"
            <button class="active" data-site-path="{playlist_site_path}" data-toggle-playlist>
                {playlist_icon} {t_playlist}
            </button>
        "#);

        actions.push(toggle_playlist);
    }

    let valid_subtitles = video.valid_subtitles().collect::<Vec<_>>();

    if !valid_subtitles.is_empty() {
        let t_transcript = translations.transcript;
        let subtitles_icon = hyper8_icons::subtitles(None);
        let toggle_transcript = formatdoc!(r#"
            <button data-toggle-transcript>
                {subtitles_icon} {t_transcript}
            </button>
        "#);

        actions.push(toggle_transcript);
    }

    if let Some(chapters) = &video.chapters && !chapters.is_empty() {
        let t_chapters = translations.chapters;
        let chapters_icon = hyper8_icons::CHAPTERS;
        let toggle_chapters = formatdoc!(r#"
            <button data-toggle-chapters>
                {chapters_icon} {t_chapters}
            </button>
        "#);

        actions.push(toggle_chapters);
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

    let mut active_panel = String::new();
    let mut panels = Vec::new();
    let mut panel_postscript = String::new();

    if let Some(playlist) = playlist {
        let playlist_context = TraversalStep::crumb_playlist(
            playlist,
            site,
            Some(video)
        );

        panels.push(playlist_context);

        let playlist_site_path = playlist.site_path.normalized();

        active_panel = String::from(r#"data-panel="playlist""#);

        // We automatically hide the playlist panel during page load if ...
        // a) it has been explicitly closed by the user before,
        //    for this or another video in the same playlist.
        // b) the viewport is so narrow that we'd display the playlist panel
        //    below the video (which would likely push the video description
        //    "below the fold" in the initially visible page - not desired).
        //    (Note that the associated media query must be kept up-to-date
        //     with the query used for the playlist panel in site.css)
        panel_postscript = formatdoc!(r#"
            <script>
                const playlistBelowVideo = matchMedia('(width < 55rem)');
                if (
                    sessionStorage.getItem('hyper8HidePlaylist:{playlist_site_path}') !== null ||
                    playlistBelowVideo.matches
                ) {{
                    const togglePlaylistButton = document.querySelector('#toggle_playlist');
                    const videoLayout = document.currentScript.previousElementSibling;
                    delete videoLayout.dataset.panel;
                    togglePlaylistButton.classList.remove('active');
                }}
            </script>
        "#);
    }

    if !valid_subtitles.is_empty() {
        let mut content = String::new();

        let active_subtitle = &valid_subtitles[0];

        for subtitle_file in &valid_subtitles {
            let mut file_subtitles = String::new();

            let subtitles = match &subtitle_file.subtitles {
                Subtitles::Invalid { .. } => unreachable!(),
                Subtitles::Valid { subtitles, .. } => subtitles
            };

            for subtitle in subtitles.iter() {
                let start_display = subtitle.start.format_display();
                let start_link = subtitle.start.format_link();
                let text_escaped = html_escape_outside_attribute(&subtitle.text);
                let subtitle_html = formatdoc!(r#"
                    <li>
                        <a href="{start_link}">
                            <span>{start_display}</span>
                            <span>{text_escaped}</span>
                        </a>
                    </li>
                "#);

                file_subtitles.push_str(&subtitle_html);
            }

            let class_active = if ptr::eq(subtitle_file, active_subtitle) {
                r#"class="active""#
            } else {
                ""
            };

            let hash = subtitle_file.subtitles.hash_unchecked();

            let file_subtitles = format!(
                r#"<ul {class_active} data-hash="{hash}">{file_subtitles}</ul>"#
            );

            content.push_str(&file_subtitles);
        }

        let track_switcher_button;
        let track_switcher_choices;

        if valid_subtitles.len() > 1 {
            let choices = valid_subtitles
                .iter()
                .map(|subtitle_file| {
                    let hash = subtitle_file.subtitles.hash_unchecked();
                    let label = html_escape_outside_attribute(&subtitle_file.label);

                    format!(r#"<button data-hash="{hash}">{label}</button>"#)
                })
                .collect::<Vec<_>>()
                .join("");

            let active_label = &active_subtitle.label;

            track_switcher_button = format!(r#"<button class="switch">{active_label}</button>"#);
            track_switcher_choices = format!(r#"<div class="switcher">{choices}</div>"#);
        } else {
            track_switcher_button = String::new();
            track_switcher_choices = String::new();
        };

        let t_filter = translations.filter;
        let transcript = formatdoc!(r#"
            <div class="transcript">
                <div class="controls">
                    <input autocomplete="off" placeholder="{t_filter}" type="search">
                    {track_switcher_button}
                </div>
                {track_switcher_choices}
                {content}
            </div>
        "#);

        panels.push(transcript);
    }

    if let Some(chapters) = &video.chapters && !chapters.is_empty() {
        let chapters = chapters
            .iter()
            .map(|chapter| {
                let seconds = chapter.begin.seconds;
                let time_range = precise_duration(seconds);
                let title = &chapter.title;

                format!(r##"<li><a href="#t={seconds}"><div>{title}</div><div>{time_range}</div></a></li>"##)
            })
            .collect::<Vec<_>>()
            .join("");

        let chapters = formatdoc!(r#"
            <div class="chapters">
                <ul>{chapters}</ul>
            </div>
        "#);

        panels.push(chapters);
    };

    let panels = panels.join("\n");

    let body = formatdoc!(r#"
        <div class="video_layout" {active_panel}>
            <div class="video_header">
                <h1 class="video_title">{e_title} {r_unlisted_badge}</h1>
                {parent_context}
            </div>
            {player_rendered}
            {panels}
            <div class="video_footer">
                <div>
                    {release_date}
                </div>
                {actions}
            </div>
            <div class="video_info">
                {links_rendered}
                {description}
            </div>
        </div>
        {panel_postscript}
    "#);

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
                // TODO: This should alternatively also utilize the default
                // audio_file, if available
                if let Some(video_file) = video.default_streamable_video_file() {
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
        build,
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

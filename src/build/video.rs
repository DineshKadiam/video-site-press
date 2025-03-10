// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

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
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_button, links, player, unlisted_badge};
use crate::icons;
use crate::util::{
    html_escape_outside_attribute,
    localized_date_d_mmmm_yyyy
};

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

    let r_downloads = if video.download() && !video.versions.is_empty() {
        let t_download = site.language.translations.download;
        let t_download_permalink = site.language.translations.download_permalink;
        let download_icon = icons::download(t_download);

        formatdoc!(r#"
            <a href="{t_download_permalink}{index_suffix}">
                {download_icon} {t_download}
            </a>
        "#)
    } else {
        String::new()
    };

    let next_video = if video.unlisted() {
        None
    } else {
        match playlist {
            Some(playlist) => {
                let videos = playlist.public_videos_in_playlist_order();

                let active_position = videos
                    .iter()
                    .position(|video_iterated| video_iterated.site_path == video.site_path)
                    .unwrap();

                videos
                    .get(active_position + 1)
                    .map(Arc::clone)
            }
            None => None
        }
    };

    // let outline_items = traversal.steps
    //     .iter()
    //     .filter(|step| !step.is_video(video))
    //     .enumerate()
    //     .map(|(index, step)| {
    //         let prefix = "../".repeat(traversal.steps.len() - 1 - index);
    //         step.context_for_video(&prefix, site, video)
    //     })
    //     .collect::<Vec<String>>()
    //     .join("\n");

    // let outline_rendered = formatdoc!(r#"
    //     <div class="outline">
    //         {outline_items}
    //     </div>
    // "#);

    let video_prefix = "";
    let player_rendered = player(next_video, site, video, video_prefix);

    let copy_link_button = if video.copy_link() {
        let (content_key, content_value) = match &site.base_url {
            Some(base_url) => ("content", base_url.join_index(site, &video.site_path)),
            None => ("dynamic-url", String::new())
        };

        let t_copy_link = site.language.translations.copy_link;
        let (button, templates) = copy_button(content_key, &content_value, t_copy_link, site);

        layout.add_clipboard_script();
        layout.add_templates(&templates);

        button
    } else {
        String::new()
    };

    let r_embed = if video.embedding() && site.base_url.is_some() && !video.versions.is_empty() {
        let embed_icon = icons::embed();

        let t_embed = &site.language.translations.embed;
        let t_embed_codes_permalink = &site.language.translations.embed_codes_permalink;

        format!(r#"
            <a href="{t_embed_codes_permalink}{index_suffix}">
                {embed_icon} {t_embed}
            </a>
        "#)
    } else {
        String::new()
    };

    let release_date = match video.release_date {
        Some(release_date) => {
            let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
            format!(r#"<div class="release_date">{formatted_date}</div>"#)
        }
        None => String::new()
    };

    let r_unlisted_badge = if video.unlisted() {
        layout.no_indexing();
        unlisted_badge(&site.language.translations)
    } else {
        String::new()
    };

    let links_rendered = links(&site.language, &video.links);

    let body = formatdoc!(r#"
        {player_rendered}
        <div class="video_info">
            <div class="align apart flex" style="margin-top: 2rem;">
                <div>
                    <h1 class="video_title">{e_title} {r_unlisted_badge}</h1>
                    {release_date}
                </div>
                <div class="actions align flex space">
                    {copy_link_button}
                    {r_embed}
                    {r_downloads}
                </div>
            </div>
            {description}
            {links_rendered}
        </div>
    "#);

    let platform_integration = video.platform_integration();
    if platform_integration.enabled() {
        if let Some(video_url) = site.absolute_url_index(&video.site_path) {
            let mut meta = PlatformIntegrationMeta::new(title.to_string(), video_url);

            if let Some(description) = &video.description {
                let plaintext = MarkdownSubset::to_plaintext(description);

                // As a somewhat arbitrary practical decision we currently
                // only use "our" description as the description for platform integration
                // when it is less than 256 characters long in plaintext.
                if plaintext.len() < 256 {
                    meta.description(description);
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
                if let Some(video_file) = video.recommended_version() {
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

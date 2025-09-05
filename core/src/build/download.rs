// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use indoc::formatdoc;

use crate::{
    Context,
    Site,
    Video,
    VideoFile
};
use crate::build::{AssetHashes, Layout};
use crate::build::outline::Traversal;
use crate::build::widgets::unlisted_badge;
use crate::util::{
    html_escape_outside_attribute,
    human_size
};

/// Writes the page on which various downloads for a video are made available
/// (video, audio and subtitle assets)
pub fn download_html(
    asset_hashes: &AssetHashes,
    context: &Context,
    site: &Site,
    video: &Video,
    traversal: &Traversal
) {
    let mut layout = Layout::new();
    let translations = &site.language.translations;

    let t_download = translations.download;
    let video_title = video.title_or_slug_or_generic_label(translations);
    let title = format!("{t_download} – {video_title}");
    let e_title = html_escape_outside_attribute(&title);

    let r_unlisted_badge = if video.unlisted() {
        layout.no_indexing();
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let mut video_files_with_meta: Vec<&VideoFile> = video.video_files
        .iter()
        .filter(|video_file| video_file.video_meta.is_some())
        .collect();

    video_files_with_meta.sort_unstable_by(|a, b| {
        a.video_meta.as_ref().unwrap().height.cmp(
            &b.video_meta.as_ref().unwrap().height
        )
    });

    let format_buttons = video_files_with_meta
        .iter()
        .map(|video_file| {
            // TODO: Clean up/better/more complete concept for how to build file names (e.g. maybe hide original names altogether!)
            let file_name = video_file.file_name.as_str();
            let video_meta = &video_file.video_meta.as_ref().unwrap();

            let height = video_meta.height;
            let width = video_meta.width;

            let download_slug = video.slug().unwrap_or("video");
            let download_name = Path::new(file_name).with_file_name(format!("{download_slug}_{width}x{height}")).to_string_lossy().to_string();

            let format_label = format!("{width}×{height}");
            let vertical_resolution_label = format!("{height}p");

            let size = human_size(video_file.file_meta.size);

            let t_no_audio = translations.no_audio;
            let audio_codec = video_meta.audio_codec.as_deref().unwrap_or(t_no_audio);

            let t_no_video = translations.no_video;
            let video_codec = video_meta.video_codec.as_deref().unwrap_or(t_no_video);

            let extension = Path::new(&video_file.file_name)
                .extension()
                .unwrap_or(std::ffi::OsStr::new(""))
                .to_string_lossy();

            formatdoc!(r#"
                <a download="{download_name}" class="align apart flex" href="../{file_name}">
                    <div>
                        <div class="large_font">{vertical_resolution_label}</div>
                        <small>{format_label}, {audio_codec}, {video_codec}, {extension}</small>
                    </div>
                    <div class="large_font">{size}</div>
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let t_subtitles = translations.subtitles;
    let subtitle_buttons = video.subtitles
        .iter()
        .map(|subtitle_file| {
            let file_name = &subtitle_file.file_name;
            let language = match &subtitle_file.language {
                Some(language_code) => format!(r#"<div class="large_font">{language_code}</div>"#),
                None => String::new()
            };

            formatdoc!(r#"
                <a download="{file_name}" class="align apart flex" href="../{file_name}">
                    <div>
                        <div class="large_font">{file_name}</div>
                        <small>{t_subtitles}</small>
                    </div>
                    {language}
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let body = format!(r#"
        <div class="video_info">
            <h1>{e_title} {r_unlisted_badge}</h1>
            {format_buttons}
            {subtitle_buttons}
        </div>
    "#);

    let html = layout.render(
        asset_hashes,
        &body,
        site,
        &title,
        traversal
    );

    let t_download_permalink = translations.download_permalink;
    let download_dir = context.build_dir
        .join(video.site_path.filesystem_relative())
        .join(t_download_permalink);

    fs::create_dir_all(&download_dir).unwrap();
    let index_path = download_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

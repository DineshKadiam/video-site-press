// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use crate::{Context, Site, Video};
use crate::build::outline::Traversal;
use crate::build::widgets::{embed_layout, player};

/// Writes html for the embed itself, i.e. what is pulled in from external
/// site B through an iframe when they embed a video from site A.
pub fn embed_html(
    context: &Context,
    site: &Site,
    video: &Video,
    traversal: &Traversal
) {
    let video_prefix = "../";
    let player_rendered = player(None, site, video, video_prefix);

    let body = format!(r#"
        {player_rendered}
    "#);

    let html = embed_layout(&body, site, traversal);

    let embed_dir = context.build_dir.join(&video.path).join("embed");
    fs::create_dir_all(&embed_dir).unwrap();
    let index_path = embed_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

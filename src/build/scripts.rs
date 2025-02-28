// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use crate::Context;

pub fn generate_adaptive_theme_js(context: &Context) {
    fs::write(
        context.build_dir.join("adaptive_theme.js"),
        include_str!("assets/adaptive_theme.js")
    ).unwrap();
}

pub fn generate_clipboard_js(context: &Context) {
    fs::write(
        context.build_dir.join("clipboard.js"),
        include_str!("assets/clipboard.js")
    ).unwrap();
}

pub fn generate_embed_js(context: &Context) {
    fs::write(
        context.build_dir.join("embed.js"),
        include_str!("assets/embed.js")
    ).unwrap();
}

pub fn generate_video_js(context: &Context) {
    fs::write(
        context.build_dir.join("video.js"),
        include_str!("assets/video.js")
    ).unwrap();
}

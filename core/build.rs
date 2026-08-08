// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hash::{DefaultHasher, Hash, Hasher};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

fn main() {
    provide_asset_hash(
        include_bytes!("src/build/assets/adaptive_theme.js"),
        "ADAPTIVE_THEME_JS"
    );

    provide_asset_hash(
        include_bytes!("src/build/assets/clipboard.js"),
        "CLIPBOARD_JS"
    );

    provide_asset_hash(
        include_bytes!("src/build/assets/embed.js"),
        "EMBED_JS"
    );

    provide_asset_hash(
        include_bytes!("src/build/assets/favicon.png"),
        "FAVICON_PNG"
    );

    provide_asset_hash(
        include_bytes!("src/build/assets/shortcuts.js"),
        "SHORTCUTS_JS"
    );

    provide_asset_hash(
        include_bytes!("src/build/assets/video.js"),
        "VIDEO_JS"
    );
}

fn provide_asset_hash(bytes: &[u8], key: &str) {
    let hash = url_safe_hash_base64(bytes);
    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_{key}={hash}");
}

pub fn url_safe_hash_base64(content: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}

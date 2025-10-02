// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hash::{DefaultHasher, Hash, Hasher};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

fn main() {
    provide_asset_hash(
        include_bytes!("assets/docs.css"),
        "DOCS_CSS"
    );

    provide_asset_hash(
        include_bytes!("assets/favicon.png"),
        "FAVICON_PNG"
    );
}

fn provide_asset_hash(bytes: &[u8], key: &str) {
    let hash = url_safe_hash_base64(bytes);
    println!("cargo:rustc-env=HYPER8_DOCS_STATIC_ASSET_HASH_{key}={hash}");
}

pub fn url_safe_hash_base64(content: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}

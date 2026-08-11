// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_util::url_safe_hash_base64;

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

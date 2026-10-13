// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

/// When we link to assets on the rendered pages, we append a unique asset
/// hash to each path (e.g. "site.js?g1VVfPoEjUw"), which is derived from
/// the file content of the asset. We do this in order to prompt browsers to
/// fetch new, uncached assets when their content has changed. This struct
/// groups together those hashes for all assets we use.
#[derive(Debug)]
pub struct AssetHashes {
    pub embed_css: Option<String>,
    pub navigation_js: Option<String>,
    pub site_css: Option<String>
}

impl AssetHashes {
    pub const ADAPTIVE_THEME_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_ADAPTIVE_THEME_JS");
    pub const CLIPBOARD_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_CLIPBOARD_JS");
    pub const EMBED_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_EMBED_JS");
    pub const FAVICON_PNG: &str = env!("HYPER8_STATIC_ASSET_HASH_FAVICON_PNG");
    pub const SHORTCUTS_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_SHORTCUTS_JS");
    pub const VIDEO_JS: &str = env!("HYPER8_STATIC_ASSET_HASH_VIDEO_JS");

    pub fn new() -> AssetHashes {
        AssetHashes {
            embed_css: None,
            navigation_js: None,
            site_css: None
        }
    }
}

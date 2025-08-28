// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Build hyper8 with (e.g.) HYPER8_VERSION=2.0.0~pre1 to override
//! the version that is displayed and reported in resulting builds.

use std::env;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::process::Command;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Provides four variables to the consecutive build step:
/// - HYPER8_REVISION (If git is available the short commit hash, otherwise "unknown revision")
/// - HYPER8_VERSION_ADAPTIVE (The version down to patch level only if it is a patch release, e.g. "1.2.1")
/// - HYPER8_VERSION_DETAILED (The version down to patch level, e.g. "1.2.0")
/// - HYPER8_VERSION_DISPLAY (The version for pretty display, without patch level, e.g. "1.2")
///
/// If HYPER8_VERSION is used to override the version, both HYPER8_VERSION_DETAILED and
/// HYPER8_VERSION_DISPLAY will contain its value, e.g. "2.0.0~pre1".
fn main() {
    let version_adaptive;
    let version_detailed;
    let version_display;
    if let Ok(override_version) = env::var("HYPER8_VERSION") {
        version_adaptive = override_version.clone();
        version_detailed = override_version.clone();
        version_display = override_version;
    } else {
        version_adaptive = match env!("CARGO_PKG_VERSION_PATCH") {
            "0" => concat!(env!("CARGO_PKG_VERSION_MAJOR"), '.', env!("CARGO_PKG_VERSION_MINOR")).to_string(),
            _ => env!("CARGO_PKG_VERSION").to_string()
        };
        version_detailed = env!("CARGO_PKG_VERSION").to_string();
        version_display = concat!(env!("CARGO_PKG_VERSION_MAJOR"), '.', env!("CARGO_PKG_VERSION_MINOR")).to_string();
    }

    let mut git = Command::new("git");
    git.args(["rev-parse", "--short", "HEAD"]);
    let revision = match git.output() {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(_) => String::from("unknown revision")
    };

    println!("cargo:rerun-if-env-changed=HYPER8_VERSION");
    println!("cargo:rustc-env=HYPER8_REVISION={revision}");
    println!("cargo:rustc-env=HYPER8_VERSION_ADAPTIVE={version_adaptive}");
    println!("cargo:rustc-env=HYPER8_VERSION_DETAILED={version_detailed}");
    println!("cargo:rustc-env=HYPER8_VERSION_DISPLAY={version_display}");

    let static_asset_hash_adaptive_theme_js = url_safe_hash_base64(include_bytes!("src/build/assets/adaptive_theme.js"));
    let static_asset_hash_clipboard_js = url_safe_hash_base64(include_bytes!("src/build/assets/clipboard.js"));
    let static_asset_hash_embed_js = url_safe_hash_base64(include_bytes!("src/build/assets/embed.js"));
    let static_asset_hash_favicon_png = url_safe_hash_base64(include_bytes!("src/build/assets/favicon.png"));
    let static_asset_hash_video_js = url_safe_hash_base64(include_bytes!("src/build/assets/video.js"));

    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_ADAPTIVE_THEME_JS={static_asset_hash_adaptive_theme_js}");
    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_CLIPBOARD_JS={static_asset_hash_clipboard_js}");
    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_EMBED_JS={static_asset_hash_embed_js}");
    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_FAVICON_PNG={static_asset_hash_favicon_png}");
    println!("cargo:rustc-env=HYPER8_STATIC_ASSET_HASH_VIDEO_JS={static_asset_hash_video_js}");
}

pub fn url_safe_hash_base64(hashable: &impl Hash) -> String {
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}

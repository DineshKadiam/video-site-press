// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Build any hyper8 target (cli, docs, etc.) with the environment variable
//! HYPER8_VERSION set to override the version that is displayed and reported
//! in resulting builds.
//!
//! For instance to build the docs for version "2.0.0~pre1":
//! "HYPER8_VERSION=2.0.0~pre1 cargo run -p hyper8-docs -- [output_path]"

/// The link https://simonrepp.com/hyper8/changes/[major.minor.patch]/ for
/// regular tag-based releases. If the version is being overridden using
/// HYPER8_VERSION, it falls back to the more generic release notes index
/// page (https://simonrepp.com/hyper8/changes/).
pub const CHANGES_URL: &str = env!("HYPER8_CHANGES_URL");

/// 7-digit short commit hash or "unknown revision"
pub const REVISION: &str = env!("HYPER8_REVISION");

/// Adaptive version, e.g. "0.23" or "0.23.1" (leaves out patch for major/minor releases)
pub const VERSION_ADAPTIVE_PATCH: &str = env!("HYPER8_VERSION_ADAPTIVE_PATCH");

/// E.g. Full version, "0.23.0" or "1.6.3"
pub const VERSION_WITH_PATCH: &str = env!("HYPER8_VERSION_WITH_PATCH");

/// E.g. "1.2.3 (0abcdef)"
pub const VERSION_WITH_PATCH_AND_REVISION: &str = concat!(
    env!("HYPER8_VERSION_WITH_PATCH"),
    " (",
    env!("HYPER8_REVISION"),
    ")"
);

/// Only the major and minor version, e.g. "0.23" or "1.6"
pub const VERSION_WITHOUT_PATCH: &str = env!("HYPER8_VERSION_WITHOUT_PATCH");

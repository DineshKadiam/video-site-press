// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Build any hyper8 target (cli, docs, etc.) with the environment variable
//! HYPER8_VERSION set to override the version that is displayed and reported
//! in resulting builds.
//!
//! For instance to build the docs for version "2.0.0~pre1":
//! "HYPER8_VERSION=2.0.0~pre1 cargo run -p hyper8-docs -- [output_path]"

/// The link https://hyper8.org/changes/[major.minor.patch]/ if a semantic
/// version is present either in the tag of the current commit (with a
/// matching version present in the cargo manifest) or passed through the
/// HYPER8_VERSION environment variable. If no semantic version is detected
/// it falls back to the more generic release notes index page at
/// https://hyper8.org/changes/.
pub const RELEASE_NOTES: &str = env!("HYPER8_RELEASE_NOTES");

/// Adaptive version, e.g. "0.23" or "0.23.1" (leaves out patch for major/minor releases)
pub const VERSION_ADAPTIVE_PATCH: &str = env!("HYPER8_VERSION_ADAPTIVE_PATCH");

/// Full version, e.g. "0.23.0" or "1.6.3"
pub const VERSION_WITH_PATCH: &str = env!("HYPER8_VERSION_WITH_PATCH");

/// Full version with 7-digit short commit hash, e.g. "1.2.3 (0abcdef)"
pub const VERSION_WITH_PATCH_AND_REVISION: &str = env!("HYPER8_VERSION_WITH_PATCH_AND_REVISION");

/// Only the major and minor version, e.g. "0.23" or "1.6"
pub const VERSION_WITHOUT_PATCH: &str = env!("HYPER8_VERSION_WITHOUT_PATCH");

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::process::Command;

/// Call git to determine the 7-digit short hash of the current revision.
/// If git is not available, we fall back to "unknown revision".
/// The output is stored in HYPER8_REVISION.
fn compute_revision() {
    let mut git = Command::new("git");

    git.args(["rev-parse", "--short", "HEAD"]);

    let revision = match git.output() {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout).unwrap(),
        _ => String::from("unknown revision")
    };

    println!("cargo:rustc-env=HYPER8_REVISION={revision}");
}

fn main() {
    compute_revision();
    compute_version();
}

/// Provides three variables to the consecutive build step:
/// - HYPER8_VERSION_ADAPTIVE_PATCH (With patch only if it is a patch release, e.g. "1.2.1")
/// - HYPER8_VERSION_WITH_PATCH (Always with patch, e.g. "1.2.0")
/// - HYPER8_VERSION_WITHOUT_PATCH (Always without patch, e.g. "1.2")
///
/// If HYPER8_VERSION is used to override the version, all three variables
/// will contain the full version, e.g. "2.0.0~pre1".
fn compute_version() {
    let version_adaptive_patch;
    let version_with_patch;
    let version_without_patch;
    if let Ok(override_version) = env::var("HYPER8_VERSION") {
        version_adaptive_patch = override_version.clone();
        version_with_patch = override_version.clone();
        version_without_patch = override_version;
    } else {
        version_adaptive_patch = if env!("CARGO_PKG_VERSION_PATCH") == "0" {
            concat!(
                env!("CARGO_PKG_VERSION_MAJOR"),
                '.',
                env!("CARGO_PKG_VERSION_MINOR")
            ).to_string()
        } else {
            env!("CARGO_PKG_VERSION").to_string()
        };

        version_with_patch = env!("CARGO_PKG_VERSION").to_string();
        version_without_patch = concat!(
            env!("CARGO_PKG_VERSION_MAJOR"),
            '.',
            env!("CARGO_PKG_VERSION_MINOR")
        ).to_string();
    }

    println!("cargo:rerun-if-env-changed=HYPER8_VERSION");
    println!("cargo:rustc-env=HYPER8_VERSION_ADAPTIVE_PATCH={version_adaptive_patch}");
    println!("cargo:rustc-env=HYPER8_VERSION_WITH_PATCH={version_with_patch}");
    println!("cargo:rustc-env=HYPER8_VERSION_WITHOUT_PATCH={version_without_patch}");
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::process::Command;

const CHANGES_INDEX_URL: &str = "https://hyper8.org/changes/";

/// For non-semantic versions we link to the release notes index page
fn export_generic_release_notes() {
    let url = CHANGES_INDEX_URL;
    export_release_notes_variable(url);
}

/// For regular semantic versions our convention is to link to release
/// notes in the "[major].[minor].[patch]/" directory under the release
/// notes index page
fn export_semantic_release_notes(
    major: &str,
    minor: &str,
    patch: &str
) {
    let url = format!("{CHANGES_INDEX_URL}{major}.{minor}.{patch}/");
    export_release_notes_variable(&url);
}

fn export_semantic_version(
    major: &str,
    minor: &str,
    patch: &str
) {
    let revision = git_revision();

    let version_adaptive_patch = if patch == "0" {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{patch}")
    };

    let version_with_patch = format!("{major}.{minor}.{patch}");
    let version_with_patch_and_revision = format!("{major}.{minor}.{patch} ({revision})");
    let version_without_patch = format!("{major}.{minor}");

    export_version_variables(
        &version_adaptive_patch,
        &version_with_patch,
        &version_with_patch_and_revision,
        &version_without_patch
    );
}

fn export_special_version(version: &str) {
    let revision = git_revision();

    let version_with_revision = if version == revision {
        version.to_string()
    } else {
        format!("{version} ({revision})")
    };

    export_version_variables(
        version,
        version,
        &version_with_revision,
        version
    );
}

fn export_release_notes_variable(url: &str) {
    println!("cargo:rustc-env=HYPER8_RELEASE_NOTES={url}");
}

fn export_version_variables(
    version_adaptive_patch: &str,
    version_with_patch: &str,
    version_with_patch_and_revision: &str,
    version_without_patch: &str
) {
    println!("cargo:rustc-env=HYPER8_VERSION_ADAPTIVE_PATCH={version_adaptive_patch}");
    println!("cargo:rustc-env=HYPER8_VERSION_WITH_PATCH={version_with_patch}");
    println!("cargo:rustc-env=HYPER8_VERSION_WITH_PATCH_AND_REVISION={version_with_patch_and_revision}");
    println!("cargo:rustc-env=HYPER8_VERSION_WITHOUT_PATCH={version_without_patch}");
}

// TODO: According to the canonical https://semver.org/ specification,
// optional additional labels containing build/pre-release metadata may only
// be separated by a narrower subset of characters than we allow (and use).
// We might want to adopt these rules eventually.

/// Extracts and returns the major, minor and patch number and optional
/// pre-release/build label from the passed version string if it follows
/// a semantic format recognized by us, such as:
/// - 1.2.3
/// - 1.2.3~beta1
/// - 1.2.3-preview1
fn extract_semantic_version(version: &str) -> Option<(&str, &str, &str, Option<&str>)> {
    let (major, remainder) = version.split_once('.')?;
    let _ = major.parse::<u16>().ok()?;

    let (minor, remainder) = remainder.split_once('.')?;
    let _ = minor.parse::<u16>().ok()?;

    match remainder.split_once(|c| c == '~' || c == '-') {
        Some((patch, label)) => {
            let _ = patch.parse::<u16>().ok()?;
            Some((major, minor, patch, Some(label)))
        }
        None => {
            let _ = remainder.parse::<u16>().ok()?;
            Some((major, minor, remainder, None))
        }
    }
}

fn git_describe() -> Option<String> {
    let mut git = Command::new("git");

    git.args(["describe"]);

    if let Ok(output) = git.output() && output.status.success() {
        Some(String::from_utf8(output.stdout).unwrap())
    } else {
        None
    }
}

/// Call git to determine the 7-digit short hash of the current revision.
/// If git is not available, we fall back to "unknown revision".
/// The output is stored in HYPER8_REVISION.
fn git_revision() -> String {
    let mut git = Command::new("git");

    git.args(["rev-parse", "--short", "HEAD"]);

    if let Ok(output) = git.output() && output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        String::from("unknown revision")
    }
}

/// Provides variables to the consecutive build step:
/// - HYPER8_RELEASE_NOTES (e.g. https://hyper8.org/changes/1.2.3/)
/// - HYPER8_VERSION_ADAPTIVE_PATCH (With patch only if it is a patch release, e.g. "1.2.3")
/// - HYPER8_VERSION_WITH_PATCH (Always with patch, e.g. "1.2.0")
/// - HYPER8_VERSION_WITH_PATCH_AND_REVISION (Always with patch and revision, e.g. "1.2.3 (0abcdef)")
/// - HYPER8_VERSION_WITHOUT_PATCH (Always without patch, e.g. "1.2")
///
/// If no semantic version is available we use various fallbacks to provide
/// some kind of version information (e.g. only the git revision).
fn main() {
    println!("cargo:rerun-if-env-changed=HYPER8_VERSION");

    if let Ok(version) = env::var("HYPER8_VERSION") {
        // Version was explicitly defined by environment variable

        if let Some((major, minor, patch, label)) = extract_semantic_version(&version) {
            export_semantic_release_notes(major, minor, patch);

            if label.is_none() {
                export_semantic_version(major, minor, patch);
            } else {
                export_special_version(&version);
            }
        } else {
            export_generic_release_notes();
            export_special_version(&version);
        }
    } else if git_describe().is_some_and(|name| name == env!("CARGO_PKG_VERSION")) {
        // Version is clearly implied by both the version declared in the
        // cargo manifest and the matching version in the tag of current
        // commit

        let major = env!("CARGO_PKG_VERSION_MAJOR");
        let minor = env!("CARGO_PKG_VERSION_MINOR");
        let patch = env!("CARGO_PKG_VERSION_PATCH");

        export_semantic_release_notes(major, minor, patch);
        export_semantic_version(major, minor, patch);
    } else {
        // We can not tell a version with any certainty, so we use the git
        // revision as a unique version identifier instead

        let revision = git_revision();

        export_generic_release_notes();
        export_special_version(&revision);
    }
}

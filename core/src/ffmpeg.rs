// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use crate::HyperCommand;

pub struct FFmpeg {
    pub bundled: bool,
    pub ffmpeg: FFmpegExecutable,
    pub ffprobe: FFmpegExecutable
}

pub struct FFmpegExecutable {
    pub executable: PathBuf,
    pub version: Option<String>
}

/// Tries to call ffmpeg and ffprobe (with -version) either as an executable
/// in the given directory, or otherwise at the system level. If it succeeds
/// for both, we assume ffmpeg and ffprobe are available and working and
/// additionally try to parse the version from the output. Parsing the
/// version might fail but is not considered an indicator of ffmpeg and
/// ffprobe being incorrectly configured (there is no stability guarantee for
/// the output of -version). We either return the ffmpeg and ffprobe
/// executable and version as Some(FFmpeg) or None if the probe failed.
fn probe_ffmpeg(executable_dir: Option<&Path>) -> Option<FFmpeg> {
    let bundled;
    let ffmpeg;
    let ffprobe;

    if let Some(executable_dir) = &executable_dir {
        bundled = true;

        ffmpeg = probe_ffmpeg_executable(
            executable_dir.join("ffmpeg"),
            "ffmpeg"
        )?;

        ffprobe = probe_ffmpeg_executable(
            executable_dir.join("ffprobe"),
            "ffprobe"
        )?;
    } else {
        bundled = false;

        ffmpeg = probe_ffmpeg_executable(
            PathBuf::from("ffmpeg"),
            "ffmpeg"
        )?;

        ffprobe = probe_ffmpeg_executable(
            PathBuf::from("ffprobe"),
            "ffprobe"
        )?;
    }

    let ffmpeg = FFmpeg {
        bundled,
        ffmpeg,
        ffprobe
    };

    Some(ffmpeg)
}

fn probe_ffmpeg_executable(
    executable: PathBuf,
    executable_name: &str
) -> Option<FFmpegExecutable> {
    let mut executable_version = HyperCommand::new(&executable);

    executable_version.arg("-version");

    if let Ok(output) = executable_version.output() && output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout);

        let version_prefix = format!("{executable_name} version ");

        let version = if let Some(remainder) = output.strip_prefix(&version_prefix) {
            let version_token = remainder.split_ascii_whitespace().next().unwrap();
            Some(version_token.to_string())
        } else {
            None
        };

        let ffmpeg_executable = FFmpegExecutable {
            executable,
            version
        };

        Some(ffmpeg_executable)
    } else {
        None
    }
}

impl FFmpeg {
    pub fn locate() -> Option<FFmpeg> {
        // Probe ffmpeg as a binary adjacent to the hyper8 binary itself. We use
        // this primarily for platforms that require bundled versions of ffmpeg to be
        // shipped with Hyper 8.
        #[cfg(any(feature = "force-bundled-ffmpeg", feature = "use-bundled-ffmpeg"))]
        {
            use std::env;

            let current_exe = env::current_exe().unwrap();
            let executable_dir = current_exe.parent().unwrap();

            let bundled_ffmpeg = probe_ffmpeg(Some(executable_dir));

            if bundled_ffmpeg.is_some() {
                return bundled_ffmpeg;
            }
        }

        // Probe ffmpeg at the system level (it needs to be in PATH)
        #[cfg(not(feature = "force-bundled-ffmpeg"))]
        {
            let system_ffmpeg = probe_ffmpeg(None);

            if system_ffmpeg.is_some() {
                return system_ffmpeg;
            }
        }

        None
    }
}

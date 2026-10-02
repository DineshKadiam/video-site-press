// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    // Make sure that on macOS desktop builds, ffmpeg/ffprobe is found
    // from $PATH - try to do without in the long run
    let _ = fix_path_env::fix();

    // If any commandline arguments are passed, hyper8 runs in
    // cli mode, returning an exit code after completion.
    if let Some(exit_code) = hyper8_cli_lib::handle_desktop() {
        return exit_code;
    }

    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    unsafe {
        // In Windows release builds we hide hyper8's console
        // window right after startup if the invocation is not
        // handled by the cli. We specifically don't use the
        // #![windows_subsystem = "windows"] top-level crate
        // attribute for this because this in turn unconditionally
        // breaks cli functionality, e.g. confirmation prompts
        // block forever (never reading from stdin) or they
        // finish instantly without blocking, etc.
        winapi::um::wincon::FreeConsole();
    }

    // If no commandline arguments were passed, hyper8 starts
    // in graphical desktop application mode.
    hyper8_desktop_lib::run();

    ExitCode::SUCCESS
}

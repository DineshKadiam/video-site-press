// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    // If any commandline arguments are passed, hyper8 runs in
    // cli mode, returning an exit code after completion.
    if let Some(exit_code) = hyper8_cli_lib::handle_desktop() {
        return exit_code;
    }

    // If no commandline arguments were passed, hyper8 starts
    // in graphical desktop application mode.
    hyper8_desktop_lib::run();

    ExitCode::SUCCESS
}

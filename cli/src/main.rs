// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    // Main entry point which begins the cli-only program flow
    hyper8_cli_lib::handle_cli_only()
}

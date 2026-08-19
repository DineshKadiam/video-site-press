// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::process::ExitCode;

use anstream::eprintln;

use hyper8_core::{Context, SyncProtocol};
use hyper8_core::check_sync_config;
use hyper8_translations::Translations;

use crate::style::{BRIGHT_BLACK, BRIGHT_YELLOW, GREEN, RED};

/// Checks the FTP or SSH sync and server configuration
pub fn check(
    context: Arc<Context>,
    protocol: SyncProtocol,
    translations: &Translations
) -> ExitCode {
    let site = context.get_site();

    if !site.complete_sync_config(protocol) {
        let message = translations.sync_configuration_is_incomplete_can_not_synchronize;
        eprintln!("{message}");
        return ExitCode::FAILURE
    }

    let steps = check_sync_config(
        protocol,
        &site,
        translations
    );

    let mut exit_code = ExitCode::SUCCESS;

    for step in steps {
        let context = if let Some(context) = &step.context {
            format!(" {context}")
        } else {
            String::new()
        };

        let label = &step.label;

        if step.failed() {
            exit_code = ExitCode::FAILURE;
            eprintln!("{RED}[FAILED]{RED:#} {label}{BRIGHT_BLACK}{context}{BRIGHT_BLACK:#}");
        } else if step.passed() {
            eprintln!("{GREEN}[PASSED]{GREEN:#} {label}{BRIGHT_BLACK}{context}{BRIGHT_BLACK:#}");
        } else /* if step.skipped() */ {
            exit_code = ExitCode::FAILURE;
            eprintln!("{BRIGHT_YELLOW}[SKIPPED]{BRIGHT_YELLOW:#} {label}{BRIGHT_BLACK}{context}{BRIGHT_BLACK:#}");
        }

        if let Some(message) = &step.message {
            eprintln!("\n{message}\n");
        }
    }

    exit_code
}

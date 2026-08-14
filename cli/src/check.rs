// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::process::ExitCode;

use hyper8_core::{Context, DeploymentProtocol};
use hyper8_core::check_deployment_config;
use hyper8_translations::Translations;

/// Checks the FTP or SSH deployment and server configuration
pub fn check(
    context: Arc<Context>,
    protocol: DeploymentProtocol,
    translations: &Translations
) -> ExitCode {
    let site = context.get_site();

    if !site.complete_deployment_config(protocol) {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy;
        eprintln!("{message}");
        return ExitCode::FAILURE
    }

    let steps = check_deployment_config(
        protocol,
        &site,
        translations
    );

    let mut exit_code = ExitCode::SUCCESS;

    for step in steps {
        match step {
            Ok(message) => {
                eprintln!("(success) {message}");
            }
            Err(message) => {
                eprintln!("(failure) {message}");
                exit_code = ExitCode::FAILURE;
            }
        }
    }

    exit_code
}

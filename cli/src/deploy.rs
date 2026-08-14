// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use hyper8_core::{
    Context,
    DeploymentProtocol,
    Site
};
use hyper8_core::start_deployment;
use hyper8_translations::Translations;

/// Performs deployment in a synchronous fashion, reporting back all messages
/// and errors to the terminal. Also asks for explicit ("opt-in") confirmation
/// before performing the actual deployment.
pub fn perform_deploy_blocking_cli(
    context: &Arc<Context>,
    explicit_protocol: Option<DeploymentProtocol>,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    let protocol = if let Some(protocol) = explicit_protocol {
        if !site.complete_deployment_config(protocol) {
            let message = translations.deployment_configuration_is_incomplete_can_not_deploy.to_string();
            return Err(message);
        }

        protocol
    } else {
        match (site.ftp_config.complete(), site.ssh_config.complete()) {
            (true, true) => {
                let message = translations.you_need_to_additionally_pass_ftp_or_ssh_text.to_string();
                return Err(message);
            }
            (true, false) => DeploymentProtocol::Ftp,
            (false, true) => DeploymentProtocol::Ssh,
            (false, false) => {
                let message = translations.no_complete_deployment_configuration_found.to_string();
                return Err(message);
            }
        }
    };

    if !simulate {
        let t_really_deploy = translations.really_deploy;
        let base_url = &site.base_url.as_ref().unwrap().representation;
        println!("{t_really_deploy} ({base_url})");
        println!("{}", translations.type_y_and_press_enter_to_confirm_anything_else_will_abort);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() != "y" {
                    return Ok(());
                }
            }
            Err(err) => return Err(err.to_string())
        }
    }

    start_deployment(
        context,
        protocol,
        simulate,
        site,
        translations
    )?;

    let mut seen_feedback = 0;

    loop {
        thread::sleep(Duration::from_secs(1));

        if let Ok(deployment) = context.deploying.lock() {
            let feedback = &deployment.feedback;

            if feedback.len() > seen_feedback {
                eprintln!("{}", &feedback[seen_feedback..]);
                seen_feedback = feedback.len();
            }

            if !deployment.active() {
                break;
            }
        }
    }

    Ok(())
}

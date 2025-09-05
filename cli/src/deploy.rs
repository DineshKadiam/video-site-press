// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use hyper8_core::{Context, Site};
use hyper8_core::{config_probe, start_deploy};
use hyper8_translations::Translations;

/// Performs deployment in a synchronous fashion, reporting back all messages
/// and errors to the terminal. Also asks for explicit ("opt-in") confirmation
/// before performing the actual deployment.
pub fn perform_deploy_blocking_cli(
    context_ref: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    if site.base_url.is_none() {
        let site_manifest_path = context_ref.site_dir.join("site.eno");
        let message = translations.public_address_must_be_configured_for_deployment_text_cli(&site_manifest_path);
        return Err(message);
    }

    if !site.complete_deployment_config() {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy.to_string();
        return Err(message);
    }

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

    if let Err(err) = config_probe(context_ref, site, translations) {
        let t_deployment_configuration_could_not_be_verified = translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} ({err})");
        return Err(message);
    }

    start_deploy(context_ref, site, translations)?;

    let mut seen_feedback = 0;

    loop {
        thread::sleep(Duration::from_secs(1));

        if let Ok(deployment) = context_ref.deploying.lock() {
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

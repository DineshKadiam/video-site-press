// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anstream::eprintln;

use hyper8_core::{
    Context,
    Site,
    SyncProtocol
};
use hyper8_core::start_sync;
use hyper8_translations::Translations;

use crate::style::{BRIGHT_BLACK, BRIGHT_YELLOW};

/// Performs live or simulated deployment in a synchronous fashion, reporting
/// back all messages and errors to the terminal. Asks for explicit
/// (opt-in) confirmation before performing a live deployment.
pub fn perform_deploy_blocking_cli(
    confirm: bool,
    context: &Arc<Context>,
    protocol: SyncProtocol,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    if !site.complete_deployment_config(protocol) {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy.to_string();
        return Err(message);
    }

    if !simulate && confirm {
        let t_really_deploy = translations.really_deploy;
        let base_url = &site.base_url.as_ref().unwrap().representation;
        eprintln!("{t_really_deploy} {BRIGHT_BLACK}{base_url}{BRIGHT_BLACK:#}");
        eprintln!("{BRIGHT_YELLOW}{}{BRIGHT_YELLOW:#}", translations.type_y_and_press_enter_to_confirm_anything_else_will_abort);

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

    start_sync(
        context,
        protocol,
        simulate,
        site,
        translations
    )?;

    let mut seen_log_range = 0;

    loop {
        thread::sleep(Duration::from_secs(1));

        if let Ok(sync_state) = context.sync_state.lock() {
            let log = &sync_state.log;

            if log.len() > seen_log_range {
                eprintln!("{}", &log[seen_log_range..]);
                seen_log_range = log.len();
            }

            if !sync_state.syncing() {
                break;
            }
        }
    }

    Ok(())
}

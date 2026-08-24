// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use anstream::eprintln;

use hyper8_core::{Site, SyncProtocol};
use hyper8_translations::Translations;

use crate::prompt;
use crate::style::RED;

pub fn ensure_config(
    non_interactive: bool,
    protocol: SyncProtocol,
    site: &Site,
    translations: &Translations
) -> Result<Option<String>, ()> {
    if !site.complete_sync_config(protocol) {
        let message = translations.sync_configuration_is_incomplete_can_not_synchronize;
        eprintln!("{RED}{message}{RED:#}");
        return Err(());
    }

    if site.prompt_sync_password(protocol) {
        if non_interactive {
            let message = translations.interactive_password_prompt_required_but_disabled_by_user;
            eprintln!("{RED}{message}{RED:#}");
            Err(())
        } else {
            let message = if protocol.ssh() {
                if let Some(private_key) = &site.ssh_config.private_key {
                    translations.enter_password_for_decrypting_the_private_key_in_xxx(
                        &private_key.to_string_lossy()
                    )
                } else {
                    let user = site.ssh_config.user_unchecked();
                    translations.enter_password_for_user_xxx(user)
                }
            } else {
                let user = site.ftp_config.user_unchecked();
                translations.enter_password_for_user_xxx(user)
            };

            eprint!("{message}: ");

            match prompt::password() {
                Ok(password) => Ok(Some(password)),
                Err(()) => Err(())
            }
        }
    } else {
        Ok(None)
    }
}

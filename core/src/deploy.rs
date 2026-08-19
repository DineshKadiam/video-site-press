// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use hyper8_translations::Translations;

use crate::{Context, Site};

mod ftp;
mod ssh;
mod sync_action;
mod sync_config_check_step;
mod sync_entry;
mod sync_error;
mod sync_protocol;
mod sync_state;


pub use ftp::FtpConfig;
pub use ssh::SshConfig;
pub use sync_action::SyncAction;
pub use sync_config_check_step::SyncConfigCheckStep;
pub use sync_entry::SyncEntry;
pub use sync_error::SyncError;
pub use sync_protocol::SyncProtocol;
pub use sync_state::{
    SyncEvent,
    SyncEventActionKind,
    SyncEventKind,
    SyncState
};

pub fn check_deployment_config(
    protocol: SyncProtocol,
    site: &Site,
    translations: &Translations
) -> Vec<SyncConfigCheckStep> {
    match protocol {
        SyncProtocol::Ftp => {
            if site.ftp_config.secure() {
                ftp::check_deployment_config(
                    ftp::connect_secure,
                    site,
                    translations
                )
            } else {
                ftp::check_deployment_config(
                    ftp::connect_insecure,
                    site,
                    translations
                )
            }
        }
        SyncProtocol::Ssh => {
            ssh::check_deployment_config(
                site,
                translations
            )
        }
    }
}

pub fn start_sync(
    context: &Arc<Context>,
    protocol: SyncProtocol,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    match protocol {
        SyncProtocol::Ftp => {
            if site.ftp_config.secure() {
                ftp::start_sync(
                    ftp::connect_secure,
                    context,
                    simulate,
                    site,
                    translations
                )
            } else {
                ftp::start_sync(
                    ftp::connect_insecure,
                    context,
                    simulate,
                    site,
                    translations
                )
            }
        }
        SyncProtocol::Ssh => {
            ssh::start_sync(
                context,
                simulate,
                site,
                translations
            )
        }
    }
}

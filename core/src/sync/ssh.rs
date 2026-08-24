// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::TcpStream;

use ssh2::{
    Session,
    Sftp
};

use hyper8_translations::Translations;

use crate::{Site, SyncError};

use super::{SyncAction, SyncEntry};

mod check;
mod ssh_config;
mod sync;

pub use check::check_sync_config;
pub use ssh_config::SshConfig;
pub use sync::start_sync;

/// SSH sync actions and entries represent modified time as 64-bit integers
/// (seconds since UNIX Epoch)
type SshSyncAction = SyncAction<u64>;
type SshSyncEntry = SyncEntry<u64>;

const DEFAULT_SSH_PORT: u16 = 22;

fn connect(
    password: Option<&str>,
    site: &Site,
    translations: &Translations
) -> Result<(Session, Sftp), SyncError> {
    let port = site.ssh_config.port.unwrap_or(DEFAULT_SSH_PORT);
    let server = site.ssh_config.server_unchecked();

    let address = format!("{server}:{port}");

    let tcp = TcpStream::connect(address)?;

    let mut session = Session::new()?;

    session.set_tcp_stream(tcp);

    session.handshake()?;

    let user = site.ssh_config.user_unchecked();

    if let Some(private_key) = &site.ssh_config.private_key {
        if let Err(err) = session.userauth_pubkey_file(
            user,
            site.ssh_config.public_key.as_deref(),
            private_key,
            password
        ) {
            let t_keyfile_authentication_failed = translations.keyfile_authentication_failed;
            let message = format!("{t_keyfile_authentication_failed}\n{err}");
            return Err(message.into());
        }
    } else if let Some(password) = password {
        if let Err(err) = session.userauth_password(user, password) {
            let t_password_authentication_failed = translations.password_authentication_failed;
            let message = format!("{t_password_authentication_failed}\n{err}");
            return Err(message.into());
        }
    } else {
        // Without password we fall back to authentication through the primary
        // identity of the system's ssh agent
        // (see https://docs.rs/ssh2/latest/ssh2/struct.Session.html#method.userauth_agent)
        if let Err(err) = session.userauth_agent(user) {
            let t_ssh_agent_authentication_failed = translations.ssh_agent_authentication_failed;
            let message = format!("{t_ssh_agent_authentication_failed}\n{err}");
            return Err(message.into());
        }
    }

    if !session.authenticated() {
        let message = translations.authentication_failed;
        return Err(SyncError(message.to_string()));
    }

    match session.sftp() {
        Ok(sftp) => Ok((session, sftp)),
        Err(err) => {
            let message = translations.sftp_subsystem_could_not_be_initialized;
            Err(SyncError(format!("{message} ({err})")))
        }
    }
}

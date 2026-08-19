// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use suppaftp::{
    FtpStream,
    NativeTlsConnector,
    NativeTlsFtpStream
};
use suppaftp::native_tls::TlsConnector;
use suppaftp::types::FileType;

use crate::{Site, SyncError};

use super::{SyncAction, SyncEntry};

mod check;
mod ftp_config;
mod sync;

pub use check::check_sync_config;
pub use ftp_config::FtpConfig;
pub use sync::start_sync;

/// FTP sync actions and entries represent modified time as date strings of
/// the form yyyymmddHHMMSS, e.g. "20250509105623"
type FtpSyncAction = SyncAction<String>;
type FtpSyncEntry = SyncEntry<String>;

const DEFAULT_FTP_PORT: u16 = 21;

pub fn connect_insecure(site: &Site) -> Result<FtpStream, SyncError> {
    let password = site.ftp_config.password_unchecked();
    let port = site.ftp_config.port.unwrap_or(DEFAULT_FTP_PORT);
    let server = site.ftp_config.server_unchecked();
    let user = site.ftp_config.user_unchecked();

    let server_and_port = format!("{server}:{port}");

    let mut stream = FtpStream::connect(server_and_port).unwrap();

    stream.login(user, password)?;
    stream.transfer_type(FileType::Binary)?;

    Ok(stream)
}

pub fn connect_secure(site: &Site) -> Result<NativeTlsFtpStream, SyncError> {
    let password = site.ftp_config.password_unchecked();
    let port = site.ftp_config.port.unwrap_or(DEFAULT_FTP_PORT);
    let server = site.ftp_config.server_unchecked();
    let user = site.ftp_config.user_unchecked();

    let server_and_port = format!("{server}:{port}");

    let stream = NativeTlsFtpStream::connect(server_and_port)?;

    let tls_connector = TlsConnector::new()
        .unwrap();

    // Uncomment and use as alternative for testing against ad-hoc
    // server setups with invalid certificates.
    // let tls_connector = TlsConnector::builder()
    //     .danger_accept_invalid_certs(true)
    //     .build()
    //     .unwrap();

    let mut stream_secure = stream.into_secure(NativeTlsConnector::from(tls_connector), server)?;

    stream_secure.login(user, password)?;
    stream_secure.transfer_type(FileType::Binary)?;

    Ok(stream_secure)
}

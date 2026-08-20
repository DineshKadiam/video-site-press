// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

/// Just a glue type that allows us to treat different types of sync-related
/// error types (suppaftp::FtpError, ssh2::Error, std::io::Error, String) as
/// one with the ? operator.
pub struct SyncError(pub String);

impl From<ssh2::Error> for SyncError {
    fn from(error: ssh2::Error) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<suppaftp::FtpError> for SyncError {
    fn from(error: suppaftp::FtpError) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<String> for SyncError {
    fn from(error: String) -> SyncError {
        SyncError(error)
    }
}

impl From<SyncError> for String {
    fn from(sync_error: SyncError) -> String {
        sync_error.0
    }
}

impl Display for SyncError {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use suppaftp::FtpError;

/// Just a glue type that allows us to treat different types of
/// sync-related error types (FtpError, std::io::Error, String)
/// as one with the ? operator.
pub struct SyncError(String);

impl From<FtpError> for SyncError {
    fn from(error: FtpError) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> SyncError {
        SyncError(error.to_string())
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

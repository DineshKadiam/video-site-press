// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug)]
pub enum SyncProtocol {
    Ftp,
    Ssh
}

impl SyncProtocol {
    pub fn ftp(&self) -> bool {
        matches!(self, SyncProtocol::Ftp)
    }

    pub fn ssh(&self) -> bool {
        matches!(self, SyncProtocol::Ssh)
    }
}

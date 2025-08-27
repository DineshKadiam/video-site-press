// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub struct SshConfig {
    pub path: Option<String>,
    pub server: Option<String>,
    pub user: Option<String>
}

impl SshConfig {
    pub fn complete(&self) -> bool {
        self.path.is_some() &&
        self.server.is_some() &&
        self.user.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_none() &&
        self.server.is_none() &&
        self.user.is_none()
    }

    pub fn new() -> SshConfig {
        SshConfig {
            path: None,
            server: None,
            user: None
        }
    }

    pub fn path_unchecked(&self) -> &str {
        self.path.as_ref().unwrap()
    }

    pub fn server_unchecked(&self) -> &str {
        self.server.as_ref().unwrap()
    }

    pub fn user_unchecked(&self) -> &str {
        self.user.as_ref().unwrap()
    }
}

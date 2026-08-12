// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub struct SshConfig {
    pub password: Option<String>,
    pub path: Option<String>,
    pub port: Option<usize>,
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
        self.password.is_none() &&
        self.path.is_none() &&
        self.port.is_none() &&
        self.server.is_none() &&
        self.user.is_none()
    }

    pub fn new() -> SshConfig {
        SshConfig {
            password: None,
            path: None,
            port: None,
            server: None,
            user: None
        }
    }

    pub fn password_unchecked(&self) -> &str {
        self.password.as_ref().unwrap()
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

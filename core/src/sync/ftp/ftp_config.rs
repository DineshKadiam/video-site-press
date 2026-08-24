// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub struct FtpConfig {
    pub password: Option<String>,
    pub password_prompt: Option<bool>,
    pub path: Option<String>,
    pub port: Option<u16>,
    // Whether to use FTP or FTPS
    pub secure: Option<bool>,
    pub server: Option<String>,
    pub user: Option<String>
}

impl FtpConfig {
    pub fn complete(&self) -> bool {
        (self.password.is_some() || self.password_prompt()) &&
        self.path.is_some() &&
        self.server.is_some() &&
        self.user.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.password.is_none() &&
        self.password_prompt.is_none() &&
        self.path.is_none() &&
        self.port.is_none() &&
        self.secure.is_none() &&
        self.server.is_none() &&
        self.user.is_none()
    }

    pub fn new() -> FtpConfig {
        FtpConfig {
            password: None,
            password_prompt: None,
            path: None,
            port: None,
            secure: None,
            server: None,
            user: None
        }
    }

    pub fn password_prompt(&self) -> bool {
        self.password_prompt.unwrap_or(false)
    }

    pub fn path_unchecked(&self) -> &str {
        self.path.as_ref().unwrap()
    }

    pub fn secure(&self) -> bool {
        self.secure.unwrap_or(true)
    }

    pub fn server_unchecked(&self) -> &str {
        self.server.as_ref().unwrap()
    }

    pub fn user_unchecked(&self) -> &str {
        self.user.as_ref().unwrap()
    }
}

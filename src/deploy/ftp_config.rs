// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub struct FtpConfig {
    pub password: Option<String>,
    pub path: Option<String>,
    pub port: Option<usize>,
    // Whether to use FTP or FTPS
    pub secure: Option<bool>,
    pub server: Option<String>,
    pub user: Option<String>
}

impl FtpConfig {
    pub fn complete(&self) -> bool {
        self.password.is_some() &&
        self.path.is_some() &&
        self.server.is_some() &&
        self.user.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.password.is_none() &&
        self.path.is_none() &&
        self.port.is_none() &&
        self.secure.is_none() &&
        self.server.is_none() &&
        self.user.is_none()
    }

    pub fn new() -> FtpConfig {
        FtpConfig {
            password: None,
            path: None,
            port: None,
            secure: None,
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

    pub fn secure(&self) -> bool {
        self.secure.is_none() || self.secure.is_some_and(|secure| secure)
    }

    pub fn server_unchecked(&self) -> &str {
        self.server.as_ref().unwrap()
    }

    pub fn user_unchecked(&self) -> &str {
        self.user.as_ref().unwrap()
    }
}

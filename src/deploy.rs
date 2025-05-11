// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::SystemTime;

use crate::{Context, Site, Translations};

mod ftp;
mod rsync;

use ftp::FtpStreamWrapper;

pub use ftp::FtpConfig;
pub use rsync::SshConfig;

pub struct DeploymentState {
    pub completed_at: Option<SystemTime>,
    pub feedback: String,
    pub started_at: Option<SystemTime>
}

impl DeploymentState {
    pub fn active(&self) -> bool {
        self.completed_at.is_none() && self.started_at.is_some()
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(SystemTime::now());
    }

    pub fn new() -> DeploymentState {
        DeploymentState {
            completed_at: None,
            feedback: String::new(),
            started_at: None
        }
    }

    pub fn report(&mut self, line: &str) {
        if !self.feedback.is_empty() {
            self.feedback.push('\n');
        }

        self.feedback.push_str(line);
    }

    pub fn start(&mut self) {
        self.completed_at = None;
        self.feedback.clear();
        self.started_at = Some(SystemTime::now());
    }
}

/// Caller needs to ensure that there is either a complete FTP or rsync
/// configuration available, otherwise will panic.
pub fn config_probe(
    context: &Context,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    // We prefer rsync-based deployment if available
    if site.ssh_config.complete() {
        return rsync::config_probe(context, site, translations);
    }

    if site.ftp_config.complete() {
        let mut ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;
        let result = ftp::config_probe(&mut ftp_stream_wrapper, site, translations);
        let _ = ftp_stream_wrapper.quit();
        return result;
    }

    unimplemented!("Caller must ensure that a complete FTP or rsync config is available")
}

/// Caller needs to ensure that there is either a complete FTP or rsync
/// configuration available, otherwise will panic.
pub fn perform_deploy(
    context_ref: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    // We prefer rsync-based deployment if available
    if site.ssh_config.complete() {
        return rsync::perform_deploy(context_ref, site, translations);
    }

    if site.ftp_config.complete() {
        let ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;
        let result = ftp::perform_deploy(context_ref, ftp_stream_wrapper, site, translations);
        return result;
    }

    unimplemented!("Caller must ensure that a complete FTP or rsync config is available")
}

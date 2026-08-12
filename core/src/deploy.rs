// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::SystemTime;

use hyper8_translations::Translations;

use crate::{Context, Site};

mod ftp;
mod ftp_config;
mod ftp_stream_wrapper;
mod ssh;
mod ssh_config;
mod sync_action;
mod sync_entry;
mod sync_error;

use ftp_stream_wrapper::FtpStreamWrapper;
use sync_action::SyncAction;
use sync_entry::SyncEntry;
use sync_error::SyncError;

pub use ftp_config::FtpConfig;
pub use ssh_config::SshConfig;

pub struct DeploymentState {
    pub completed_at: Option<SystemTime>,
    // TODO: It would be beneficial to have this as a less raw resource,
    // e.g. a stack (Vec) where we push separate String entries, enriched
    // with a timestamp of when they were reported?
    // This would allow for more sophisticated feedback reporting in the editor
    // as well as in the CLI.
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

/// Caller needs to ensure that there is either a complete FTP or SSH
/// configuration available, otherwise will panic.
pub fn config_probe(
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    // We prefer SSH-based deployment if available
    if site.ssh_config.complete() {
        return ssh::config_probe(site, translations);
    }

    if site.ftp_config.complete() {
        let mut ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;
        let result = ftp::config_probe(&mut ftp_stream_wrapper, site, translations);
        let _ = ftp_stream_wrapper.quit();
        return result;
    }

    panic!("Caller must ensure that a complete FTP or SSH config is available")
}

/// Caller needs to ensure that there is either a complete FTP or SSH
/// configuration available, otherwise will panic.
pub fn start_deploy(
    context: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    // We prefer SSH-based deployment if available
    if site.ssh_config.complete() {
        return ssh::start_deploy(context, site, translations);
    }

    if site.ftp_config.complete() {
        let ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;
        let result = ftp::start_deploy(context, ftp_stream_wrapper, site, translations);
        return result;
    }

    panic!("Caller must ensure that a complete FTP or SSH config is available")
}

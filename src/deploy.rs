// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::{Context, Site, Translations};

mod ftp;
mod ftp_config;
mod ftp_stream_wrapper;
mod ftp_sync_error;
mod rsync;
mod ssh_config;

use ftp_stream_wrapper::FtpStreamWrapper;
use ftp_sync_error::SyncError;

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

    panic!("Caller must ensure that a complete FTP or rsync config is available")
}

/// Performs deployment in a synchronous fashion, reporting back all messages
/// and errors to the terminal. Also asks for explicit ("opt-in") confirmation
/// before performing the actual deployment.
pub fn perform_deploy_blocking_cli(
    context_ref: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    if site.base_url.is_none() {
        let site_manifest_path = context_ref.site_dir.join("site.eno");
        let message = translations.public_address_must_be_configured_for_deployment_text_cli(&site_manifest_path);
        return Err(message);
    }

    if !site.complete_deployment_config() {
        let message = translations.deployment_configuration_is_incomplete_can_not_deploy.to_string();
        return Err(message);
    }

    let t_really_deploy = translations.really_deploy;
    let base_url = &site.base_url.as_ref().unwrap().representation;
    println!("{t_really_deploy} ({base_url})");
    println!("{}", translations.type_y_and_press_enter_to_confirm_anything_else_will_abort);

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if input.trim() != "y" {
                return Ok(());
            }
        }
        Err(err) => return Err(err.to_string())
    }

    if let Err(err) = config_probe(context_ref, site, translations) {
        let t_deployment_configuration_could_not_be_verified = translations.deployment_configuration_could_not_be_verified;
        let message = format!("{t_deployment_configuration_could_not_be_verified} ({err})");
        return Err(message);
    }

    start_deploy(context_ref, site, translations)?;

    let mut seen_feedback = 0;

    loop {
        thread::sleep(Duration::from_secs(1));

        if let Ok(deployment) = context_ref.deploying.lock() {
            let feedback = &deployment.feedback;

            if feedback.len() > seen_feedback {
                eprintln!("{}", &feedback[seen_feedback..]);
                seen_feedback = feedback.len();
            }

            if !deployment.active() {
                break;
            }
        }
    }

    Ok(())
}

/// Caller needs to ensure that there is either a complete FTP or rsync
/// configuration available, otherwise will panic.
pub fn start_deploy(
    context_ref: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    // We prefer rsync-based deployment if available
    if site.ssh_config.complete() {
        return rsync::start_deploy(context_ref, site, translations);
    }

    if site.ftp_config.complete() {
        let ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;
        let result = ftp::start_deploy(context_ref, ftp_stream_wrapper, site, translations);
        return result;
    }

    panic!("Caller must ensure that a complete FTP or rsync config is available")
}

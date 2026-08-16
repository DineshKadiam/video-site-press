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

const DEFAULT_FTP_PORT: u16 = 21;
const DEFAULT_SSH_PORT: u16 = 22;

#[derive(Debug)]
pub struct DeploymentCheckStep {
    /// Additional short, inlineable info such as the user, server, directory used
    /// for the respective step
    pub context: Option<String>,
    /// Optional long (single or multi-line) message containing success or
    /// error details
    pub message: Option<String>,
    result: DeploymentCheckStepResult,
    pub subject: String
}

#[derive(Debug)]
pub struct DeploymentCheckStepBuilder {
    pub context: Option<String>,
    pub message: Option<String>,
    pub subject: String
}

#[derive(Debug)]
enum DeploymentCheckStepResult {
    Failed,
    Passed,
    Skipped
}

#[derive(Clone, Copy, Debug)]
pub enum DeploymentProtocol {
    Ftp,
    Ssh
}

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

impl DeploymentCheckStep {
    pub fn failed(&self) -> bool {
        matches!(self.result, DeploymentCheckStepResult::Failed)
    }

    pub fn new(subject: impl Into<String>) -> DeploymentCheckStepBuilder {
        let subject = subject.into();

        DeploymentCheckStepBuilder {
            context: None,
            message: None,
            subject
        }
    }

    pub fn passed(&self) -> bool {
        matches!(self.result, DeploymentCheckStepResult::Passed)
    }

    pub fn skipped(&self) -> bool {
        matches!(self.result, DeploymentCheckStepResult::Skipped)
    }
}

impl DeploymentCheckStepBuilder {
    pub fn context(&mut self, context: impl Into<String>) -> &DeploymentCheckStepBuilder {
        self.context = Some(context.into());
        self
    }

    pub fn fail(self) -> DeploymentCheckStep {
        DeploymentCheckStep {
            context: self.context,
            message: self.message,
            result: DeploymentCheckStepResult::Failed,
            subject: self.subject
        }
    }

    pub fn message(&mut self, message: impl Into<String>) -> &DeploymentCheckStepBuilder {
        self.message = Some(message.into());
        self
    }

    pub fn pass(self) -> DeploymentCheckStep {
        DeploymentCheckStep {
            context: self.context,
            message: self.message,
            result: DeploymentCheckStepResult::Passed,
            subject: self.subject
        }
    }

    pub fn skip(self) -> DeploymentCheckStep {
        DeploymentCheckStep {
            context: self.context,
            message: self.message,
            result: DeploymentCheckStepResult::Skipped,
            subject: self.subject
        }
    }
}

impl DeploymentProtocol {
    pub fn ftp(&self) -> bool {
        match self {
            DeploymentProtocol::Ftp => true,
            DeploymentProtocol::Ssh => false
        }
    }

    pub fn ssh(&self) -> bool {
        match self {
            DeploymentProtocol::Ftp => false,
            DeploymentProtocol::Ssh => true
        }
    }
}

impl DeploymentState {
    pub fn active(&self) -> bool {
        self.completed_at.is_none() && self.started_at.is_some()
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(SystemTime::now());
    }

    pub fn completed(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn duration_secs_unchecked(&self) -> f32 {
        let duration = if let Some(completed_at) = &self.completed_at {
            completed_at.duration_since(self.started_at.unwrap()).unwrap()
        } else {
            self.started_at.unwrap().elapsed().unwrap()
        };

        duration.as_secs_f32()
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

pub fn check_deployment_config(
    protocol: DeploymentProtocol,
    site: &Site,
    translations: &Translations
) -> Vec<DeploymentCheckStep> {
    match protocol {
        DeploymentProtocol::Ftp => {
            ftp::check_deployment_config(
                site,
                translations
            )
        }
        DeploymentProtocol::Ssh => {
            ssh::check_deployment_config(
                site,
                translations
            )
        }
    }
}

pub fn start_deployment(
    context: &Arc<Context>,
    protocol: DeploymentProtocol,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    match protocol {
        DeploymentProtocol::Ftp => {
            ftp::start_deployment(
                context,
                simulate,
                site,
                translations
            )
        }
        DeploymentProtocol::Ssh => {
            ssh::start_deployment(
                context,
                simulate,
                site,
                translations
            )
        }
    }
}

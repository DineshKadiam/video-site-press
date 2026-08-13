// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::SystemTime;

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

pub use ftp::{start_ftp_deployment, verify_ftp_deployment_config};
pub use ftp_config::FtpConfig;
pub use ssh::{start_ssh_deployment, verify_ssh_deployment_config};
pub use ssh_config::SshConfig;

#[derive(Clone, Copy, Debug)]
pub enum DeploymentMode {
    Ftp,
    FtpSimulated,
    Ssh,
    SshSimulated
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

impl DeploymentMode {
    pub fn ftp(&self) -> bool {
        match self {
            DeploymentMode::Ftp => true,
            DeploymentMode::FtpSimulated => true,
            DeploymentMode::Ssh => false,
            DeploymentMode::SshSimulated => false
        }
    }

    pub fn new(protocol: DeploymentProtocol, simulate: bool) -> DeploymentMode {
        match protocol {
            DeploymentProtocol::Ftp => if simulate { DeploymentMode::FtpSimulated } else { DeploymentMode::Ftp },
            DeploymentProtocol::Ssh => if simulate { DeploymentMode::SshSimulated } else { DeploymentMode::Ssh }
        }
    }

    pub fn simulate(&self) -> bool {
        match self {
            DeploymentMode::Ftp => false,
            DeploymentMode::FtpSimulated => true,
            DeploymentMode::Ssh => false,
            DeploymentMode::SshSimulated => true
        }
    }

    pub fn ssh(&self) -> bool {
        match self {
            DeploymentMode::Ftp => false,
            DeploymentMode::FtpSimulated => false,
            DeploymentMode::Ssh => true,
            DeploymentMode::SshSimulated => true
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

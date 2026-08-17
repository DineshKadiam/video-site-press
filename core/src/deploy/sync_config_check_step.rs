// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug)]
pub struct SyncConfigCheckStep {
    /// Additional short, inlineable info such as the user, server, directory used
    /// for the respective step
    pub context: Option<String>,
    pub label: String,
    /// Optional long (single or multi-line) message containing success or
    /// error details
    pub message: Option<String>,
    result: SyncConfigCheckStepResult
}

#[derive(Debug)]
pub struct SyncConfigCheckStepBuilder {
    pub context: Option<String>,
    pub message: Option<String>,
    pub label: String
}

#[derive(Debug)]
enum SyncConfigCheckStepResult {
    Failed,
    Passed,
    Skipped
}

impl SyncConfigCheckStep {
    pub fn failed(&self) -> bool {
        matches!(self.result, SyncConfigCheckStepResult::Failed)
    }

    pub fn new(label: impl Into<String>) -> SyncConfigCheckStepBuilder {
        let label = label.into();

        SyncConfigCheckStepBuilder {
            context: None,
            label,
            message: None
        }
    }

    pub fn passed(&self) -> bool {
        matches!(self.result, SyncConfigCheckStepResult::Passed)
    }

    pub fn skipped(&self) -> bool {
        matches!(self.result, SyncConfigCheckStepResult::Skipped)
    }
}

impl SyncConfigCheckStepBuilder {
    pub fn context(&mut self, context: impl Into<String>) -> &SyncConfigCheckStepBuilder {
        self.context = Some(context.into());
        self
    }

    pub fn fail(self) -> SyncConfigCheckStep {
        SyncConfigCheckStep {
            context: self.context,
            label: self.label,
            message: self.message,
            result: SyncConfigCheckStepResult::Failed
        }
    }

    pub fn message(&mut self, message: impl Into<String>) -> &SyncConfigCheckStepBuilder {
        self.message = Some(message.into());
        self
    }

    pub fn pass(self) -> SyncConfigCheckStep {
        SyncConfigCheckStep {
            context: self.context,
            label: self.label,
            message: self.message,
            result: SyncConfigCheckStepResult::Passed
        }
    }

    pub fn skip(self) -> SyncConfigCheckStep {
        SyncConfigCheckStep {
            context: self.context,
            label: self.label,
            message: self.message,
            result: SyncConfigCheckStepResult::Skipped
        }
    }
}

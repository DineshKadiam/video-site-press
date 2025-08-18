// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_derive::Serialize;

use crate::{JobProgress, SitePath};
use crate::util::timestamp;

/// An internal numerical identifier that is associated with
/// user-facing job information (label, site path, progress)
#[derive(Clone, Serialize)]
pub struct ActiveJob {
    pub id: usize,
    pub label: String,
    pub last_progress: Option<JobProgress>,
    pub site_path: SitePath
}

/// Keeps track of all jobs that are currently being processed
#[derive(Clone, Serialize)]
pub struct ActiveJobs {
    pub jobs: Vec<ActiveJob>,
    pub last_update: u128
}

impl ActiveJobs {
    /// Add a job to the active jobs and return an id with which it can be
    /// identified consecutively.
    pub fn add(&mut self, label: String, site_path: SitePath) -> usize {
        let id = self.jobs
            .iter()
            .last()
            .map(|job| job.id + 1)
            .unwrap_or(1);

        let job = ActiveJob {
            id,
            label,
            last_progress: None,
            site_path,
        };

        self.last_update = timestamp();
        self.jobs.push(job);

        id
    }

    pub fn new() -> ActiveJobs {
        ActiveJobs {
            jobs: Vec::new(),
            last_update: timestamp()
        }
    }

    pub fn remove(&mut self, id: usize) {
        let index_option = self.jobs
            .iter()
            .position(|job| job.id == id);

        if let Some(index) = index_option {
            self.last_update = timestamp();
            self.jobs.remove(index);
        }
    }

    pub fn update_progress(&mut self, id: usize, progress: JobProgress) {
        if let Some(job) = self.jobs
            .iter_mut()
            .find(|job| job.id == id) {
            self.last_update = timestamp();
            job.last_progress = Some(progress);
        }
    }
}

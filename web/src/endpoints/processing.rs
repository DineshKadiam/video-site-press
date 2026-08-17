// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::web::{Data, Json, Path as ActixPath};
use serde_derive::Serialize;

use hyper8_core::{JobProgress, SitePath, SyncState};

use crate::Context;

/// Represents the state of a running deployment at request time
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentInfo {
    /// Duration of last completed deployment
    deployed_in: Option<f32>,
    /// Duration of ongoing deployment in seconds
    deploying_since: Option<f32>,
    feedback: String
}

/// Represents the state of a set of jobs (either all or a local subset) at
/// request time
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobReport {
    pub active_jobs: Vec<Option<JobProgress>>,
    pub completed_count: usize,
    pub failed_count: usize,
    pub pending_count: usize
}

/// Returned as (camel-cased) JSON through the /processing endpoint. The
/// global job report feeds the live processing feedback displayed in the
/// header, while the local job report optionally provides specific live job
/// information for media processing of a video.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub deployment_info: DeploymentInfo,
    pub global_job_report: JobReport,
    pub local_job_report: Option<JobReport>
}

impl DeploymentInfo {
    pub fn new(sync_state: &SyncState) -> DeploymentInfo {
        let deployed_in;
        let deploying_since;

        if sync_state.syncing() {
            deployed_in = None;
            deploying_since = Some(sync_state.duration_secs_unchecked());
        } else if sync_state.finished() {
            deployed_in = Some(sync_state.duration_secs_unchecked());
            deploying_since = None;
        } else {
            deployed_in = None;
            deploying_since = None;
        };

        DeploymentInfo {
            deployed_in,
            deploying_since,
            feedback: sync_state.log.clone()
        }
    }
}

impl JobReport {
    pub fn new() -> JobReport {
        JobReport {
            active_jobs: Vec::new(),
            completed_count: 0,
            failed_count: 0,
            pending_count: 0
        }
    }
}

impl Report {
    pub fn new(
        deployment_info: DeploymentInfo,
        global_job_report: JobReport,
        local_job_report: Option<JobReport>
    ) -> Report {
        Report {
            deployment_info,
            global_job_report,
            local_job_report
        }
    }
}

pub async fn processing_general(
    context: Data<Arc<Context>>
) -> Json<Report> {
    processing(&context, None)
}

pub async fn processing_media(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Json<Report> {
    processing(&context, Some(&site_path))
}

fn processing(
    context: &Context,
    site_path: Option<&SitePath>
) -> Json<Report> {
    let deployment_info = {
        let sync_state = context.sync_state.lock().unwrap();
        DeploymentInfo::new(&sync_state)
    };

    let mut global_job_report = JobReport::new();
    let mut local_job_report = match site_path.is_some() {
        true => Some(JobReport::new()),
        false => None
    };

    {
        let job_queue = context.job_queue.lock().unwrap();

        for job in job_queue.iter() {
            // Register all jobs
            if let Some(assignment) = &job.active_assignment {
                global_job_report.active_jobs.push(assignment.last_progress.clone());
            } else {
                if job.failure.is_some() {
                    global_job_report.failed_count += 1;
                } else {
                    global_job_report.pending_count += 1;
                }
            }

            // Register media specific jobs, if requested
            if let Some(local_job_report) = &mut local_job_report {
                if job.desc.targets_media(site_path.unwrap()) {
                    if let Some(assignment) = &job.active_assignment {
                        local_job_report.active_jobs.push(assignment.last_progress.clone());
                    } else {
                        if job.failure.is_some() {
                            local_job_report.failed_count += 1;
                        } else {
                            local_job_report.pending_count += 1;
                        }
                    }
                }
            }
        }
    }

    {
        let completed_jobs = context.completed_jobs.lock().unwrap();

        for job in completed_jobs.iter() {
            // Register all jobs
            global_job_report.completed_count += 1;

            // Register media specific jobs, if requested
            if let Some(local_job_report) = &mut local_job_report {
                if job.desc.targets_media(site_path.unwrap()) {
                    local_job_report.completed_count += 1;
                }
            }
        }
    }

    let report = Report::new(
        deployment_info,
        global_job_report,
        local_job_report
    );

    Json(report)
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::web::{Data, Json};
use serde_derive::Serialize;

use hyper8_core::{DeploymentState, JobProgress};

use crate::Context;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentInfo {
    configured: bool,
    /// Duration of ongoing deployment in seconds
    deploying_since: Option<f32>,
    feedback: String
}

/// Field names get camel-cased for JSON serialization and subsequent
/// transmission over the /processing endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub active_jobs: Vec<Option<JobProgress>>,
    pub deployment_info: DeploymentInfo,
    pub failed_count: usize,
    pub pending_count: usize
}

impl DeploymentInfo {
    pub fn configured(deployment_state: &DeploymentState) -> DeploymentInfo {
        let deploying_since = if deployment_state.active() {
            let duration = deployment_state.started_at
                .unwrap()
                .elapsed()
                .map(|duration| duration.as_secs_f32())
                .unwrap_or(0.0);

            Some(duration)
        } else {
            None
        };

        DeploymentInfo {
            configured: true,
            deploying_since,
            feedback: deployment_state.feedback.clone()
        }
    }

    pub fn not_configured() -> DeploymentInfo {
        DeploymentInfo {
            configured: false,
            deploying_since: None,
            feedback: String::new()
        }
    }
}

impl Report {
    pub fn new(
        active_jobs: Vec<Option<JobProgress>>,
        deployment_info: DeploymentInfo,
        failed_count: usize,
        pending_count: usize
    ) -> Report {
        Report {
            active_jobs,
            deployment_info,
            failed_count,
            pending_count
        }
    }
}

pub async fn processing(context: Data<Arc<Context>>) -> Json<Report> {
    let report = {
        let deployment_info = {
            let site = context.get_site();

            if site.complete_deployment_config() {
                let deployment_state = context.deploying.lock().unwrap();
                DeploymentInfo::configured(&deployment_state)
            } else {
                DeploymentInfo::not_configured()
            }
        };

        let mut failed_count = 0;
        let mut pending_count = 0;

        let active_jobs = {
            let job_queue = context.job_queue.lock().unwrap();

            job_queue
                .iter()
                .filter_map(|job| {
                    if let Some(assignment) = &job.active_assignment {
                        Some(assignment.last_progress.clone())
                    } else {
                        if job.failure.is_some() {
                            failed_count += 1;
                        } else {
                            pending_count += 1;
                        }

                        None
                    }
                })
                .collect()
        };

        Report::new(
            active_jobs,
            deployment_info,
            failed_count,
            pending_count
        )
    };

    Json(report)
}

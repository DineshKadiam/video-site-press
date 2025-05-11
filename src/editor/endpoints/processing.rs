// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::web::{Data, Json};
use serde_derive::Serialize;

use crate::{Context, DeploymentState, Progress};

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
    pub deployment_info: DeploymentInfo,
    pub last_update: u128,
    pub operations: Vec<Option<Progress>>,
    pub pending_count: usize
}

impl DeploymentInfo {
    pub fn configured(deployment_state: &DeploymentState) -> DeploymentInfo {
        let deploying_since = if deployment_state.active() {
            let duration = deployment_state.started_at.unwrap().elapsed().map(|duration| duration.as_secs_f32()).unwrap_or(0.0);
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
        deployment_info: DeploymentInfo,
        last_update: u128,
        operations: Vec<Option<Progress>>,
        pending_count: usize
    ) -> Report {
        Report {
            deployment_info,
            last_update,
            operations,
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

        let (last_update, operations) = {
            let processing = context.processing.lock().unwrap();

            let operations = processing.operations
                .iter()
                .map(|operation| operation.last_progress.clone())
                .collect();

            (processing.last_update, operations)
        };

        let pending_count = {
            let job_queue = context.job_queue.lock().unwrap();
            job_queue.iter().filter(|job| job.assigned_at.is_none()).count()
        };

        Report::new(deployment_info, last_update, operations, pending_count)
    };

    Json(report)
}

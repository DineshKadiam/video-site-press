use std::sync::Arc;

use actix_web::web::{Data, Json};
use serde_derive::Serialize;

use crate::{Context, Progress};

/// Field names get camel-cased for JSON serialization and subsequent
/// transmission over the /processing endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Meanings of the value:
    /// - Some((true, n)) = Deploying since n seconds
    /// - Some((false, _)) = Not deploying
    /// - None = Not configured
    pub deploying: Option<(bool, f32)>,
    pub last_update: u128,
    pub operations: Vec<Option<Progress>>,
    pub pending_count: usize
}

impl Report {
    pub fn new(
        deploying: Option<(bool, f32)>,
        last_update: u128,
        operations: Vec<Option<Progress>>,
        pending_count: usize
    ) -> Report {
        Report {
            deploying,
            last_update,
            operations,
            pending_count
        }
    }
}

pub async fn processing(context: Data<Arc<Context>>) -> Json<Report> {
    let report = {
        let deploying = {
            let site = context.get_site();

            if site.deploy_config.complete() {
                let deployment = context.deploying.lock().unwrap();

                if deployment.active() {
                    let duration = deployment.started_at.unwrap().elapsed().map(|duration| duration.as_secs_f32()).unwrap_or(0.0);
                    Some((true, duration))
                } else {
                    Some((false, 0.0))
                }
            } else {
                None
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

        Report::new(deploying, last_update, operations, pending_count)
    };

    Json(report)
}
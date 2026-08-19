// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::web::{Data, Json, Path as ActixPath};
use chrono::DateTime;
use chrono::offset::Local;
use serde_derive::Serialize;

use hyper8_core::{
    JobProgress,
    SitePath,
    SyncEvent,
    SyncEventActionKind,
    SyncEventKind,
    SyncState
};
use hyper8_util::human_size_delta;

use crate::Context;

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
    pub global_job_report: JobReport,
    pub local_job_report: Option<JobReport>,
    pub sync_state: Option<SyncStateJson>
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEventJson {
    pub kind: &'static str,
    pub message: String,
    pub started_at: String,
    pub step: Option<u32>,
    pub size: Option<String>
}

/// Represents the state of a running deployment at request time
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStateJson {
    /// Duration of the ongoing or finished sync
    duration: f32,
    finished: bool,
    log: Vec<SyncEventJson>,
    session_id: String,
    simulation: bool,
    steps: Option<u32>
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
        global_job_report: JobReport,
        local_job_report: Option<JobReport>,
        sync_state: Option<SyncStateJson>
    ) -> Report {
        Report {
            global_job_report,
            local_job_report,
            sync_state
        }
    }
}

impl From<&SyncEvent> for SyncEventJson {
    fn from(sync_event: &SyncEvent) -> SyncEventJson {
        let kind;
        let message;
        let size;

        match &sync_event.kind {
            SyncEventKind::Action { kind: action_kind, path } => {
                match action_kind {
                    SyncEventActionKind::AddFile { size_delta } => {
                        kind = "add_file";
                        size = Some(human_size_delta(*size_delta));
                    }
                    SyncEventActionKind::CreateDir => {
                        kind = "create_dir";
                        size = None;
                    }
                    SyncEventActionKind::DeleteDir => {
                        kind = "delete_dir";
                        size = None;
                    }
                    SyncEventActionKind::DeleteFile { size_delta } => {
                        kind = "delete_file";
                        size = Some(human_size_delta(*size_delta));
                    }
                    SyncEventActionKind::UpdateFile { size_delta } => {
                        kind = "update_file";
                        size = Some(human_size_delta(*size_delta));
                    }
                };

                message = path.to_string();
            }
            SyncEventKind::Error(error_message) => {
                kind = "error";
                message = error_message.to_string();
                size = None;
            }
            SyncEventKind::Generic(generic_message) => {
                kind = "generic";
                message = generic_message.to_string();
                size = None;
            }
        }

        let started_at_datetime: DateTime<Local> = sync_event.started_at.into();
        let started_at = started_at_datetime.format("%T").to_string();
        let step = sync_event.step;

        SyncEventJson {
            kind,
            message,
            started_at,
            step,
            size
        }
    }
}

impl From<&SyncState> for SyncStateJson {
    fn from(sync_state: &SyncState) -> SyncStateJson {
        let duration = sync_state.duration_secs();
        let finished = sync_state.finished();
        let session_id = sync_state.session_id.clone();
        let simulation = sync_state.simulation;
        let steps = sync_state.steps;

        let log = sync_state.log
            .iter()
            .map(|event| event.into())
            .collect::<Vec<_>>();

        SyncStateJson {
            duration,
            finished,
            log,
            session_id,
            simulation,
            steps,
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
    let sync_state = context.sync_state.lock().unwrap()
        .as_ref()
        .map(|sync_state| sync_state.into());

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
        global_job_report,
        local_job_report,
        sync_state
    );

    Json(report)
}

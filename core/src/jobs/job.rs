// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

use crate::util::nanoid;

use super::{JobDesc, JobError, JobProgress};

/// A background process that has already completed (successfully), including
/// all relevant data for display: When it was created, started, completed,
/// and what it was about (job description).
#[derive(Clone, Debug)]
pub struct CompletedJob {
    pub completed_at: SystemTime,
    pub created_at: SystemTime,
    pub desc: JobDesc,
    pub started_at: SystemTime
}

/// Relevant data for a job that is actively being processed
#[derive(Clone, Debug)]
pub struct JobAssignment {
    pub last_progress: Option<JobProgress>,
    pub started_at: SystemTime,
    pub worker_number: usize
}

/// Data attached to a job that has failed
#[derive(Clone, Debug)]
pub struct JobFailure {
    pub failed_at: SystemTime,
    pub started_at: SystemTime,
    pub error: JobError
}

/// A QueuedJob tracks some form of background process throughout its
/// lifecycle before completion, including when it was created/started, what
/// task it contains, whether it failed and why, a unique id, etc.
#[derive(Clone, Debug)]
pub struct QueuedJob {
    pub active_assignment: Option<JobAssignment>,
    pub created_at: SystemTime,
    pub desc: JobDesc,
    pub failure: Option<JobFailure>,
    pub id: String
}

impl CompletedJob {
    /// How long the job took to complete
    pub fn duration(&self) -> Duration {
        self.completed_at.duration_since(self.started_at).unwrap()
    }
}

impl JobAssignment {
    pub fn new(worker_number: usize) -> JobAssignment {
        JobAssignment {
            last_progress: None,
            started_at: SystemTime::now(),
            worker_number
        }
    }
}

impl JobFailure {
    /// How long the job took before it failed
    pub fn duration(&self) -> Duration {
        self.failed_at.duration_since(self.started_at).unwrap()
    }
}

impl QueuedJob {
    pub fn assign(&mut self, worker_number: usize) {
        let job_assignment = JobAssignment::new(worker_number);

        self.active_assignment = Some(job_assignment);
    }

    /// Determine the priority-based ordering between this and another queued job.
    /// Metadata and poster computation (FastTasks) is most prioritized,
    /// followed by video transcoding, followed by audio encoding. For prioritization between
    /// different formats see cmp_priority() implementation on VideoFormat. If
    /// two jobs have the same priority based on their description (and format, where
    /// relevant), the one that exists longer is prioritized.
    pub fn cmp_priority(&self, other: &QueuedJob) -> Ordering {
        match &self.desc {
            JobDesc::AudioFormat { format, .. } => match &other.desc {
                JobDesc::AudioFormat { format: other_format, .. } => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                },
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } |
                JobDesc::VideoFormat { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Less
            }
            JobDesc::AudioMeta { .. } => match other.desc {
                JobDesc::AudioFormat { .. } |
                JobDesc::VideoFormat { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } => ()
            }
            JobDesc::CollectionFastTasks { .. } => match other.desc {
                JobDesc::AudioFormat { .. } |
                JobDesc::VideoFormat { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } => ()
            }
            JobDesc::PlaylistFastTasks { .. } => match other.desc {
                JobDesc::AudioFormat { .. } |
                JobDesc::VideoFormat { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } => ()
            }
            JobDesc::VideoFastTasks { .. } => match other.desc {
                JobDesc::AudioFormat { .. } |
                JobDesc::VideoFormat { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } => ()
            }
            JobDesc::VideoFormat { format, .. } => match &other.desc {
                JobDesc::AudioFormat { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } |
                JobDesc::VideoPickPoster { .. } => return Ordering::Less,
                JobDesc::VideoFormat { format: other_format, .. } => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                }
            }
            JobDesc::VideoPickPoster { .. } => match other.desc {
                JobDesc::AudioFormat { .. } |
                JobDesc::VideoFormat { .. } => return Ordering::Greater,
                JobDesc::AudioMeta { .. } |
                JobDesc::CollectionFastTasks { .. } |
                JobDesc::PlaylistFastTasks { .. } |
                JobDesc::VideoFastTasks { .. } => return Ordering::Less,
                JobDesc::VideoPickPoster { .. } => ()
            }
        }

        self.created_at.cmp(&other.created_at).reverse()
    }

    pub fn complete(self) -> CompletedJob {
        CompletedJob {
            completed_at: SystemTime::now(),
            created_at: self.created_at,
            desc: self.desc,
            started_at: self.active_assignment.unwrap().started_at
        }
    }

    pub fn fail(&mut self, error: JobError) {
        let active_assignment = self.active_assignment.take().unwrap();

        let failure = JobFailure {
            started_at: active_assignment.started_at,
            failed_at: SystemTime::now(),
            error
        };

        self.failure = Some(failure);
    }

    pub fn new(desc: JobDesc) -> QueuedJob {
        QueuedJob {
            active_assignment: None,
            created_at: SystemTime::now(),
            desc,
            failure: None,
            id: nanoid()
        }
    }
}

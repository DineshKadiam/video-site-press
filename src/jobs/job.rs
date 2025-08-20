// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

use crate::JobDesc;
use crate::util::nanoid;

/// A job tracks some form of background process throughout its lifecycle.
/// (when it was created/started/completed, what task it contains, a unique id)
#[derive(Clone, Debug)]
pub struct Job {
    pub assigned_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub created: SystemTime,
    pub desc: JobDesc,
    pub error: Option<String>,
    pub id: String
}

impl Job {
    /// Determine the priority-based ordering between this and another job.
    /// Metadata and poster computation (FastTasks) is most prioritized,
    /// followed by video transcoding, followed by audio encoding. For prioritization between
    /// different formats see cmp_priority() implementation on VideoFormat. If
    /// two jobs have the same priority based on their kind (and format, where
    /// relevant), the one that exists longer is prioritized.
    pub fn cmp_priority(&self, other: &Job) -> Ordering {
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

        self.created.cmp(&other.created).reverse()
    }

    // How long the job took
    pub fn duration(&self) -> Option<Duration> {
        match self.assigned_at {
            Some(assigned_at) => match self.completed_at {
                Some(completed_at) => match completed_at.duration_since(assigned_at) {
                    Ok(duration) => Some(duration),
                    Err(_) => None
                }
                None => None
            }
            None => None
        }
    }

    pub fn new(desc: JobDesc) -> Job {
        Job {
            assigned_at: None,
            completed_at: None,
            created: SystemTime::now(),
            desc,
            error: None,
            id: nanoid()
        }
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

use crate::JobKind;
use crate::util::nanoid;

/// A job tracks some form of background process throughout its lifecycle.
/// (when it was created/started/completed, what task it contains, a unique id)
#[derive(Clone, Debug)]
pub struct Job {
    pub assigned_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub created: SystemTime,
    pub id: String,
    pub kind: JobKind
}

impl Job {
    /// Determine the priority-based ordering between this and another job.
    /// Metadata and poster computation (FastTasks) is most prioritized,
    /// followed by video transcoding, followed by audio encoding. For prioritization between
    /// different formats see cmp_priority() implementation on VideoFormat. If
    /// two jobs have the same priority based on their kind (and format, where
    /// relevant), the one that exists longer is prioritized.
    pub fn cmp_priority(&self, other: &Job) -> Ordering {
        match &self.kind {
            JobKind::AudioFormat { format, .. } => match &other.kind {
                JobKind::AudioFormat { format: other_format, .. } => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                },
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } |
                JobKind::VideoFormat { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Less
            }
            JobKind::AudioMeta { .. } => match other.kind {
                JobKind::AudioFormat { .. } |
                JobKind::VideoFormat { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } => ()
            }
            JobKind::CollectionFastTasks(_) => match other.kind {
                JobKind::AudioFormat { .. } |
                JobKind::VideoFormat { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } => ()
            }
            JobKind::PlaylistFastTasks(_) => match other.kind {
                JobKind::AudioFormat { .. } |
                JobKind::VideoFormat { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } => ()
            }
            JobKind::VideoFastTasks { .. } => match other.kind {
                JobKind::AudioFormat { .. } |
                JobKind::VideoFormat { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } => ()
            }
            JobKind::VideoFormat { format, .. } => match &other.kind {
                JobKind::AudioFormat { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } |
                JobKind::VideoPickPoster { .. } => return Ordering::Less,
                JobKind::VideoFormat { format: other_format, .. } => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                }
            }
            JobKind::VideoPickPoster { .. } => match other.kind {
                JobKind::AudioFormat { .. } |
                JobKind::VideoFormat { .. } => return Ordering::Greater,
                JobKind::AudioMeta { .. } |
                JobKind::CollectionFastTasks(_) |
                JobKind::PlaylistFastTasks(_) |
                JobKind::VideoFastTasks { .. } => return Ordering::Less,
                JobKind::VideoPickPoster { .. } => ()
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

    pub fn new(kind: JobKind) -> Job {
        Job {
            assigned_at: None,
            completed_at: None,
            created: SystemTime::now(),
            id: nanoid(),
            kind
        }
    }
}

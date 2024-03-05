use std::cmp::Ordering;
use std::sync::Arc;

use serde_derive::Serialize;

use crate::{Video, VideoFormat};
use crate::util::{nanoid, timestamp};

#[derive(Clone, Debug)]
pub struct Job {
    pub assigned: bool,
    pub created: u128,
    pub id: String,
    pub kind: JobKind,
    pub video: Arc<Video>
}

#[derive(Clone, Debug)]
pub enum JobKind {
    FastTasks,
    Format(VideoFormat)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: usize,
    pub label: String,
    pub video_path: String
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Processing {
    pub last_update: u128,
    pub operations: Vec<Operation>
}

impl Job {
    pub fn new(kind: JobKind, video: Arc<Video>) -> Job {
        Job {
            assigned: false,
            created: timestamp(),
            id: nanoid(),
            kind,
            video
        }
    }

    /// Determine the priority-based ordering between this and another job.
    /// Metadata and poster computation (FastTasks) is most prioritized,
    /// followed by transcoding to different formats. For prioritization between
    /// different formats see cmp_priority() implementation on VideoFormat. If
    /// two jobs have the same priority based on their kind (and format, where
    /// relevant), the one that exists longer is prioritized.
    pub fn cmp_priority(&self, other: &Job) -> Ordering {
        match &self.kind {
            JobKind::FastTasks => match other.kind {
                JobKind::FastTasks => (),
                JobKind::Format(_) => return Ordering::Greater
            }
            JobKind::Format(format) => match &other.kind {
                JobKind::FastTasks => return Ordering::Less,
                JobKind::Format(other_format) => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                }
            }
        }

        self.created.cmp(&other.created).reverse()
    }
}

impl Processing {
    pub fn new() -> Processing {
        Processing {
            last_update: timestamp(),
            operations: Vec::new()
        }
    }

    /// Add a background operation to the processing registry, return an id
    /// with which it can be identified in the registry again.
    pub fn push(&mut self, video_path: &str, label: String) -> usize {
        let id = self.operations
            .iter()
            .last()
            .map(|operation| operation.id + 1)
            .unwrap_or(1);

        println!("{label}");

        let operation = Operation {
            video_path: video_path.to_owned(),
            id,
            label
        };

        self.last_update = timestamp();
        self.operations.push(operation);

        id
    }

    pub fn remove(&mut self, id: usize) {
        let index_option = self.operations
            .iter()
            .position(|operation| operation.id == id);

        if let Some(index) = index_option {
            self.last_update = timestamp();
            self.operations.remove(index);
        }
    }
}
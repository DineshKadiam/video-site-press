use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde_derive::Serialize;

use crate::{Context, Playlist, Video, VideoFormat};
use crate::util::{nanoid, timestamp};

#[derive(Clone, Debug)]
pub struct Job {
    pub assigned_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub created: u128,
    pub id: String,
    pub kind: JobKind
}

// This specifies what needs to be done in a processing job
#[derive(Clone, Debug)]
pub enum JobKind {
    PlaylistFastTasks(Arc<Playlist>),
    VideoFastTasks {
        poster_aspect: f32,
        video: Arc<Video>
    },
    VideoFormat {
        format: VideoFormat,
        video: Arc<Video>
    },
    VideoRandomPoster {
        poster_aspect: f32,
        video: Arc<Video>
    }
}

/// Used for the /processing JSON endpoint
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: usize,
    pub label: String,
    pub path: String
}

/// Used for the /processing JSON endpoint
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Processing {
    pub last_update: u128,
    pub operations: Vec<Operation>
}

impl Job {
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
            created: timestamp(),
            id: nanoid(),
            kind
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
            JobKind::PlaylistFastTasks(_) => match other.kind {
                JobKind::PlaylistFastTasks(_) => (),
                JobKind::VideoFastTasks { .. } => (),
                JobKind::VideoFormat { .. } => return Ordering::Greater,
                JobKind::VideoRandomPoster { .. } => return Ordering::Greater
            }
            JobKind::VideoFastTasks { .. } => match other.kind {
                JobKind::PlaylistFastTasks(_) => (),
                JobKind::VideoFastTasks { .. } => (),
                JobKind::VideoFormat { .. } => return Ordering::Greater,
                JobKind::VideoRandomPoster { .. } => return Ordering::Greater
            }
            JobKind::VideoFormat { format, .. } => match &other.kind {
                JobKind::PlaylistFastTasks(_) => return Ordering::Less,
                JobKind::VideoFastTasks { .. } => return Ordering::Less,
                JobKind::VideoFormat { format: other_format, .. } => match format.cmp_priority(other_format) {
                    Ordering::Equal => (),
                    ordering => return ordering
                },
                JobKind::VideoRandomPoster { .. } => return Ordering::Less
            }
            JobKind::VideoRandomPoster { .. } => match other.kind {
                JobKind::PlaylistFastTasks(_) => return Ordering::Less,
                JobKind::VideoFastTasks { .. } => return Ordering::Less,
                JobKind::VideoFormat { .. } => return Ordering::Greater,
                JobKind::VideoRandomPoster { .. } => ()
            }
        }

        self.created.cmp(&other.created).reverse()
    }
}

impl JobKind {
    pub fn representation(&self, context: &Context) -> String {
        match self {
            JobKind::PlaylistFastTasks(playlist) => format!("Compute banner for playlist '{}'", playlist.label_editor(context)),
            JobKind::VideoFastTasks { video, .. } => format!("Compute metadata and/or poster for video '{}'", video.label_editor(context)),
            JobKind::VideoFormat { video, .. } => format!("Compute additional format for video '{}'", video.label_editor(context)),
            JobKind::VideoRandomPoster { video, .. } => format!("Compute random poster for video '{}'", video.label_editor(context))
        }
    }
}

impl PartialEq for JobKind {
    fn eq(&self, other: &JobKind) -> bool {
        match self {
            JobKind::PlaylistFastTasks(playlist) => match other {
                JobKind::PlaylistFastTasks(other_playlist) => playlist.path == other_playlist.path,
                _ => false
            }
            JobKind::VideoFastTasks { poster_aspect, video } => match other {
                JobKind::VideoFastTasks { poster_aspect: other_poster_aspect, video: other_video } => {
                    video.path == other_video.path && poster_aspect == other_poster_aspect
                }
                _ => false
            }
            JobKind::VideoFormat { video, .. } => match other {
                JobKind::VideoFormat { video: other_video, .. } => video.path == other_video.path,
                _ => false
            }
            JobKind::VideoRandomPoster { video, .. } => match other {
                JobKind::VideoRandomPoster { video: other_video, .. } => video.path == other_video.path,
                _ => false
            }
        }
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
    pub fn push(&mut self, path: &str, label: String) -> usize {
        let id = self.operations
            .iter()
            .last()
            .map(|operation| operation.id + 1)
            .unwrap_or(1);

        println!("{label}");

        let operation = Operation {
            path: path.to_owned(),
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
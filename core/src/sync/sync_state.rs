// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::SystemTime;

use hyper8_util::nanoid;

#[derive(Clone, Debug)]
pub struct SyncEvent {
    pub kind: SyncEventKind,
    pub started_at: SystemTime,
    pub step: Option<u32>
}

#[derive(Clone, Debug)]
pub enum SyncEventActionKind {
    AddFile {
        size_delta: i64
    },
    CreateDir,
    DeleteDir,
    DeleteFile {
        size_delta: i64
    },
    UpdateFile {
        size_delta: i64
    }
}

#[derive(Clone, Debug)]
pub enum SyncEventKind {
    Action {
        kind: SyncEventActionKind,
        path: String
    },
    Error(String),
    Generic(String)
}

#[derive(Debug)]
pub struct SyncState {
    pub finished_at: Option<SystemTime>,
    pub log: Vec<SyncEvent>,
    pub session_id: String,
    pub started_at: SystemTime,
    pub steps: Option<u32>,
    pub simulation: bool
}

impl SyncEvent {
    pub fn new(kind: SyncEventKind, step: Option<u32>) -> SyncEvent {
        let started_at = SystemTime::now();

        SyncEvent {
            kind,
            started_at,
            step
        }
    }
}

impl SyncState {
    pub fn duration_secs(&self) -> f32 {
        let duration = if let Some(finished_at) = &self.finished_at {
            finished_at.duration_since(self.started_at).unwrap()
        } else {
            self.started_at.elapsed().unwrap()
        };

        duration.as_secs_f32()
    }

    pub fn finished(&self) -> bool {
        self.finished_at.is_some()
    }

    pub fn log(&mut self, event_kind: SyncEventKind) {
        let event = SyncEvent::new(event_kind.into(), None);
        self.log.push(event);
    }

    pub fn log_step(&mut self, event_kind: SyncEventKind, step: u32) {
        let event = SyncEvent::new(event_kind.into(), Some(step));
        self.log.push(event);
    }

    pub fn new(simulation: bool) -> SyncState {
        let started_at = SystemTime::now();
        let session_id = nanoid();

        SyncState {
            finished_at: None,
            log: Vec::new(),
            session_id,
            started_at,
            steps: None,
            simulation
        }
    }

    pub fn set_finished(&mut self) {
        self.finished_at = Some(SystemTime::now());
    }

    pub fn set_steps(&mut self, steps: u32) {
        self.steps = Some(steps);
    }
}

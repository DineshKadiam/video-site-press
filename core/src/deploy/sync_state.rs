// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::SystemTime;

pub struct SyncState {
    pub finished_at: Option<SystemTime>,
    // TODO: It would be beneficial to have this as a less raw resource, e.g.
    // a stack (Vec) where we push separate String entries, enriched with a
    // timestamp of when they were reported? This would allow for more
    // sophisticated log reporting in the editor as well as in the CLI.
    pub log: String,
    pub started_at: Option<SystemTime>
}

impl SyncState {
    pub fn duration_secs_unchecked(&self) -> f32 {
        let duration = if let Some(finished_at) = &self.finished_at {
            finished_at.duration_since(self.started_at.unwrap()).unwrap()
        } else {
            self.started_at.unwrap().elapsed().unwrap()
        };

        duration.as_secs_f32()
    }

    pub fn finished(&self) -> bool {
        self.finished_at.is_some()
    }

    pub fn log(&mut self, line: &str) {
        if !self.log.is_empty() {
            self.log.push('\n');
        }

        self.log.push_str(line);
    }

    pub fn new() -> SyncState {
        SyncState {
            finished_at: None,
            log: String::new(),
            started_at: None
        }
    }

    pub fn set_finished(&mut self) {
        self.finished_at = Some(SystemTime::now());
    }

    pub fn set_started(&mut self) {
        self.finished_at = None;
        self.log.clear();
        self.started_at = Some(SystemTime::now());
    }

    pub fn syncing(&self) -> bool {
        self.finished_at.is_none() && self.started_at.is_some()
    }
}

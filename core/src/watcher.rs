// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use notify::{
    Config,
    EventKind,
    RecommendedWatcher,
    RecursiveMode,
    Watcher
};

use crate::Context;

const DEBOUNCE_PERIOD_MILLIS: u64 = 500;

/// Watches the site directory on disk for changes (e.g. such that were
/// externally done in a file manager by the user) and triggers updates to
/// the internal site tree.
pub fn watch_site_dir(context: Arc<Context>) {
    thread::spawn(move || {
        let (sender, receiver) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(sender, Config::default()).unwrap();
        watcher.watch(&context.site_dir, RecursiveMode::Recursive).unwrap();

        let mut last_change = Instant::now();
        let mut queue: Vec<PathBuf> = Vec::new();

        let queue_push = |last_change: &mut Instant, path: &Path, queue: &mut Vec<PathBuf>| {
            // Only queue the path if we do not already have it queued and 
            // if it is not contained within a path we already have queued.
            if !queue.iter().any(|iterated| path.starts_with(iterated)) {
                // Unqueue any paths that are contained within the path we are adding to the queue
                *queue = queue.drain(..).filter(|iterated| iterated.starts_with(path)).collect::<Vec<PathBuf>>();

                queue.push(path.to_owned());
            }
            *last_change = Instant::now();
        };

        loop {
            if let Ok(received) = receiver.recv_timeout(Duration::from_millis(DEBOUNCE_PERIOD_MILLIS)) {
                match received {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Create(_) |
                            EventKind::Modify(_) |
                            EventKind::Remove(_) => {
                                for path in &event.paths {
                                    // Discard hidden directories and files
                                    if path.iter().any(|component| component.to_string_lossy().starts_with('.')) {
                                        continue
                                    }

                                    // If it's a file (e.g. a manifest, image or video) we update the containing folder.
                                    // If it's a folder (e.g. a video, playlist or collection) we update the containing folder.
                                    // In short, we always update the parent folder!
                                    if let Some(parent) = path.parent() {
                                        queue_push(&mut last_change, parent, &mut queue);
                                    }
                                }
                            }
                            EventKind::Access(_) |
                            EventKind::Any |
                            EventKind::Other => ()
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e)
                }
            }

            if last_change.elapsed().as_millis() > DEBOUNCE_PERIOD_MILLIS as u128 {
                if !queue.is_empty() {
                    let site = context.get_site();

                    for path in queue.drain(..) {
                        context.update_from_filesystem(&path, &site);
                    }

                    context.compute_job_queue();
                }

                last_change = Instant::now();
            }
        }
    });
}

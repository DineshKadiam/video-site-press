// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use super::SyncEntry;

pub enum SyncAction<M> {
    AddFile {
        local_entry: SyncEntry<M>,
        remote_path: PathBuf
    },
    CreateDir { remote_path: PathBuf },
    DeleteDir { remote_path: PathBuf },
    DeleteFile { remote_entry: SyncEntry<M> },
    PrereqDeleteDir { remote_path: PathBuf },
    PrereqDeleteFile { remote_entry: SyncEntry<M> },
    UpdateFile {
        local_entry: SyncEntry<M>,
        remote_entry: SyncEntry<M>,
        /// During syncing we want some files to be updated later, or even
        /// at the latest possible point. For instance, updating the RSS feed
        /// files should only happen as the very last step, as everything
        /// that the feed points to should be uploaded and in order the
        /// moment that feed subscribers receive an update! This index is
        /// derived from the file extensions and helps us to order files
        /// accordingly.
        type_based_retardation_index: usize
    }
}

impl<M> Debug for SyncAction<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncAction::AddFile { local_entry, remote_path } => {
                write!(f, "SyncAction::AddFile [{}, {}]", remote_path.display(), local_entry.size)
            }
            SyncAction::CreateDir { remote_path } => {
                write!(f, "SyncAction::CreateDir [{}]", remote_path.display())
            }
            SyncAction::DeleteDir { remote_path } => {
                write!(f, "SyncAction::DeleteDir [{}]", remote_path.display())
            }
            SyncAction::DeleteFile { remote_entry } => {
                write!(f, "SyncAction::DeleteFile [{}, {}]", remote_entry.path.display(), remote_entry.size)
            }
            SyncAction::PrereqDeleteDir { remote_path } => {
                write!(f, "SyncAction::PrereqDeleteDir [{}]", remote_path.display())
            }
            SyncAction::PrereqDeleteFile { remote_entry } => {
                write!(f, "SyncAction::PrereqDeleteFile [{} {}]", remote_entry.path.display(), remote_entry.size)
            }
            SyncAction::UpdateFile { local_entry, remote_entry, .. } => {
                write!(f, "SyncAction::UpdateFile [{}, {}]", remote_entry.path.display(), local_entry.size)
            }
        }
    }
}

impl<M> SyncAction<M> {
    /// Establishes an ordering for sync actions that prioritizes the
    /// avoidance of glitches/service interruptions for the site visitor, at
    /// the cost of temporary storage usage spikes.
    ///
    /// 1. Delete all files that need to be deleted as a prerequisite to later actions, going from biggest to smallest
    /// 2. Delete all directories that need to be deleted as a prerequisite to later actions, going from deepest to shallowest
    /// 3. Create all missing directories, going from shallowest to deepest
    /// 4. Upload all completely new files, going from biggest to smallest
    /// 5. Replace updated files, going from biggest to smallest
    /// 6. Delete all files, from biggest to smallest
    /// 7. Delete all directories, going from deepest to shallowest
    pub fn minimal_service_interruptions_sort(
        a: &SyncAction<M>,
        b: &SyncAction<M>
    ) -> Ordering {
        match a {
            SyncAction::AddFile { local_entry: a_local_entry, .. } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Greater,
                    SyncAction::CreateDir { .. } => Ordering::Greater,
                    SyncAction::AddFile { local_entry: b_local_entry, .. } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_local_entry.size.cmp(&a_local_entry.size)
                    }
                    SyncAction::UpdateFile { .. } => Ordering::Less,
                    SyncAction::DeleteFile { .. } => Ordering::Less,
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
            SyncAction::CreateDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Greater,
                    SyncAction::CreateDir { remote_path: b_remote_path } => {
                        // Shallower directories are created earlier, deeper directories later
                        // (Create /foo -> Create /foo/bar -> Create /foo/bar/baz)
                        a_remote_path.components().count().cmp(&b_remote_path.components().count())
                    }
                    SyncAction::AddFile { .. } => Ordering::Less,
                    SyncAction::UpdateFile { .. } => Ordering::Less,
                    SyncAction::DeleteFile { .. } => Ordering::Less,
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
            SyncAction::DeleteDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Greater,
                    SyncAction::CreateDir { .. } => Ordering::Greater,
                    SyncAction::AddFile { .. } => Ordering::Greater,
                    SyncAction::UpdateFile { .. } => Ordering::Greater,
                    SyncAction::DeleteFile { .. } => Ordering::Greater,
                    SyncAction::DeleteDir { remote_path: b_remote_path } => {
                        // Deeper directories are deleted earlier, shallower directories later
                        // (Delete /foo/bar/baz -> Delete /foo/bar -> Delete /foo)
                        b_remote_path.components().count().cmp(&a_remote_path.components().count())
                    }
                }
            }
            SyncAction::DeleteFile { remote_entry: a_remote_entry } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Greater,
                    SyncAction::CreateDir { .. } => Ordering::Greater,
                    SyncAction::AddFile { .. } => Ordering::Greater,
                    SyncAction::UpdateFile { .. } => Ordering::Greater,
                    SyncAction::DeleteFile { remote_entry: b_remote_entry } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_remote_entry.size.cmp(&a_remote_entry.size)
                    }
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
            SyncAction::PrereqDeleteDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { remote_path: b_remote_path } => {
                        // Deeper directories are deleted earlier, shallower directories later
                        // (Delete /foo/bar/baz -> Delete /foo/bar -> Delete /foo)
                        b_remote_path.components().count().cmp(&a_remote_path.components().count())
                    }
                    SyncAction::CreateDir { .. } => Ordering::Less,
                    SyncAction::AddFile { .. } => Ordering::Less,
                    SyncAction::UpdateFile { .. } => Ordering::Less,
                    SyncAction::DeleteFile { .. } => Ordering::Less,
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
            SyncAction::PrereqDeleteFile { remote_entry: a_remote_entry } => {
                match b {
                    SyncAction::PrereqDeleteFile { remote_entry: b_remote_entry } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_remote_entry.size.cmp(&a_remote_entry.size)
                    }
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Less,
                    SyncAction::CreateDir { .. } => Ordering::Less,
                    SyncAction::AddFile { .. } => Ordering::Less,
                    SyncAction::UpdateFile { .. } => Ordering::Less,
                    SyncAction::DeleteFile { .. } => Ordering::Less,
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
            SyncAction::UpdateFile {
                local_entry: a_local_entry,
                type_based_retardation_index: a_type_based_retardation_index,
                ..
            } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => Ordering::Greater,
                    SyncAction::PrereqDeleteDir { .. } => Ordering::Greater,
                    SyncAction::CreateDir { .. } => Ordering::Greater,
                    SyncAction::AddFile { .. } => Ordering::Greater,
                    SyncAction::UpdateFile {
                        local_entry: b_local_entry,
                        type_based_retardation_index: b_type_based_retardation_index,
                        ..
                    } => {
                        a_type_based_retardation_index.cmp(b_type_based_retardation_index)
                            // Bigger files are updated earlier, smaller files later
                            // (Update Gigabytes -> Update Megabytes -> Update Kilobytes)
                            .then(b_local_entry.size.cmp(&a_local_entry.size))
                    }
                    SyncAction::DeleteFile { .. } => Ordering::Less,
                    SyncAction::DeleteDir { .. } => Ordering::Less
                }
            }
        }
    }

    // TODO: Implement
    // Tight storage quota strategy (prioritize keeping used storage as low as possible at every point of the sync)
    // "Prioritize avoidance of storage quota overflows during syncing"
    //
    // 1. Create all missing directories (recursively - going from shallowest to deepest path)
    // 2. Delete all files, from biggest to smallest
    // 3. Delete all directories (recursively - going from deepest to shallowest path)
    // 4. Replace updated files, going from biggest to smallest
    // 5. Upload all completely NEW files, going from biggest to smallest
    // pub fn minimal_storage_peaks_sort(
    //  a: &SyncAction,
    //  b: &SyncAction
    // ) -> Ordering {
    //     todo!()
    // }
}

// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::UNIX_EPOCH;

use ssh2::{
    DisconnectCode,
    FileStat,
    Sftp
};

use hyper8_translations::Translations;
use hyper8_util::{string_from_os_str, string_from_os_string};

use crate::{Context, Site, SyncError};

use super::{SshSyncAction, SshSyncEntry};
use super::connect;

/// Register `local_dir` and all recursively contained dirs and files for addition
fn collect_add_actions(
    actions: &mut Vec<SshSyncAction>,
    local_dir: SshSyncEntry,
    remote_containing_dir: &Path
) -> Result<(), SyncError> {
    // Add containing dir
    let remote_dir = remote_containing_dir.join(&local_dir.file_name);
    actions.push(SshSyncAction::CreateDir { remote_path: remote_dir.clone() });

    let local_child_entries = get_local_entries(false, &local_dir.path)?;

    for local_entry in local_child_entries {
        if local_entry.is_dir {
            // Create child dir
            collect_add_actions(actions, local_entry, &remote_dir)?;
        } else {
            // Add child file
            let remote_path = remote_dir.join(&local_entry.file_name);
            actions.push(SshSyncAction::AddFile { local_entry, remote_path });
        }
    }

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for deletion
fn collect_delete_actions(
    actions: &mut Vec<SshSyncAction>,
    remote_dir: SshSyncEntry,
    sftp: &Sftp
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(false, &remote_dir.path, sftp)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_delete_actions(actions, remote_entry, sftp)?;
        } else {
            // Delete child file
            actions.push(SshSyncAction::DeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(SshSyncAction::DeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for
/// prerequisite deletion
fn collect_prerequisite_delete_actions(
    actions: &mut Vec<SshSyncAction>,
    remote_dir: SshSyncEntry,
    sftp: &Sftp
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(false, &remote_dir.path, sftp)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_prerequisite_delete_actions(actions, remote_entry, sftp)?;
        } else {
            // Delete child file
            actions.push(SshSyncAction::PrereqDeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(SshSyncAction::PrereqDeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Compare local and remote directories and files and register the respective
/// syncing actions (add, replace, recurse, remove, etc.) for each
fn collect_sync_actions(
    actions: &mut Vec<SshSyncAction>,
    // When we call this at the root/entry level of our sync operation we
    // ignore hidden files, as they might be owned by the server
    // infrastructure and should not be meddled with
    // (e.g. ".ftpquota" or ".htaccess" files)
    ignore_hidden: bool,
    local_dir: &Path,
    remote_dir: &Path,
    sftp: &Sftp
) -> Result<(), SyncError> {
    let local_entries = get_local_entries(ignore_hidden, local_dir)?;
    let mut remote_entries = get_remote_entries(ignore_hidden, remote_dir, sftp)?;

    for local_entry in local_entries {
        let remote_entry_with_same_name = remote_entries
            .iter()
            .position(|remote_entry| remote_entry.file_name == local_entry.file_name)
            .map(|index| remote_entries.swap_remove(index));

        match remote_entry_with_same_name {
            Some(remote_entry) => {
                if local_entry.is_dir {
                    if remote_entry.is_dir {
                        // Recurse into subdir
                        collect_sync_actions(
                            actions,
                            false,
                            &local_entry.path,
                            &remote_entry.path,
                            sftp
                        )?;
                    } else {
                        // Replace remote file with dir
                        actions.push(SshSyncAction::PrereqDeleteFile { remote_entry });
                        collect_add_actions(actions, local_entry, remote_dir)?;
                    }
                } else if remote_entry.is_dir {
                    // Replace remote dir with file
                    collect_prerequisite_delete_actions(actions, remote_entry, sftp)?;

                    // Add new file to the remote
                    let remote_path = remote_dir.join(&local_entry.file_name);
                    actions.push(SshSyncAction::AddFile { local_entry, remote_path });
                } else if remote_entry.modified != local_entry.modified {
                    let type_based_retardation_index = remote_entry.type_based_retardation_index();

                    // Replace remote file with new file
                    let action = SshSyncAction::UpdateFile {
                        local_entry,
                        remote_entry,
                        type_based_retardation_index
                    };
                    actions.push(action);
                }
            }
            None => {
                if local_entry.is_dir {
                    collect_add_actions(actions, local_entry, remote_dir)?;
                } else {
                    // Add new file to the remote
                    let remote_path = remote_dir.join(&local_entry.file_name);
                    actions.push(SshSyncAction::AddFile { local_entry, remote_path });
                }
            }
        };
    }

    // Register leftover remote entries that don't correspond to anything we
    // have locally for deletion, recursively for all child dirs too.
    for remote_entry in remote_entries {
        if remote_entry.is_dir {
            // Delete remote dir and all children, recursively
            collect_delete_actions(actions, remote_entry, sftp)?;
        } else {
            // Delete remote file
            let action = SshSyncAction::DeleteFile { remote_entry };
            actions.push(action);
        }
    }

    Ok(())
}

/// The order of passed sync actions must be correct and sensible (!), i.e.
/// directories to be created that contain other directories need to be
/// created before their children, etc.
fn execute_sync_actions(
    actions: &[SshSyncAction],
    log: &mut impl FnMut(&str),
    sftp: &Sftp,
    simulate: bool,
    translations: &Translations
) -> Result<(), SyncError> {
    for action in actions {
        match action {
            SshSyncAction::AddFile { local_entry, remote_path } => {
                let message = translations.adding_file_xxx(&remote_path.to_string_lossy());
                log(&message);

                if !simulate {
                    write_entry(local_entry, remote_path, sftp)?;
                }
            }
            SshSyncAction::CreateDir { remote_path } => {
                let message = translations.creating_directory_xxx(&remote_path.to_string_lossy());
                log(&message);

                if !simulate {
                    // 775 (octal) = rwxr-xr-x (7 = rwx (owner), 5 = rx (group/others))
                    let mode = 0o755;

                    sftp.mkdir(remote_path, mode)?;
                }
            }
            SshSyncAction::DeleteDir { remote_path } |
            SshSyncAction::PrereqDeleteDir { remote_path } => {
                let message = translations.deleting_directory_xxx(&remote_path.to_string_lossy());
                log(&message);

                if !simulate {
                    sftp.rmdir(remote_path)?;
                }
            }
            SshSyncAction::DeleteFile { remote_entry } |
            SshSyncAction::PrereqDeleteFile { remote_entry } => {
                let message = translations.deleting_file_xxx(&remote_entry.path.to_string_lossy());
                log(&message);

                if !simulate {
                    sftp.unlink(&remote_entry.path)?;
                }
            }
            SshSyncAction::UpdateFile { local_entry, remote_entry, .. } => {
                let message = translations.updating_file_xxx(&remote_entry.path.to_string_lossy());
                log(&message);

                if !simulate {
                    write_entry(local_entry, &remote_entry.path, sftp)?;
                }
            }
        }
    }

    Ok(())
}

/// Constructs a dir/file listing with all relevant data points for syncing
/// from a local dir. `local_dir` must be an absolute path.
fn get_local_entries(
    ignore_hidden: bool,
    local_dir: &Path
) -> Result<Vec<SshSyncEntry>, SyncError> {
    let mut local_entries = Vec::new();

    for dir_entry_result in fs::read_dir(local_dir)? {
        let dir_entry = dir_entry_result?;
        let metadata = dir_entry.metadata()?;
        let is_dir = metadata.is_dir();

        if is_dir || metadata.is_file() {
            let file_name = string_from_os_string(dir_entry.file_name());

            if ignore_hidden && file_name.starts_with('.') { continue }

            let modified_system_time = metadata.modified().unwrap();
            let modified = modified_system_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

            let path = dir_entry.path();
            let size = if is_dir { 0 } else { metadata.len() };

            let sync_entry = SshSyncEntry {
                file_name,
                is_dir,
                modified,
                path,
                size
            };

            local_entries.push(sync_entry);
        }
    }

    Ok(local_entries)
}

/// Constructs a dir/file listing with all relevant data points for syncing
/// from a remote dir. `remote_dir` must be an absolute path.
fn get_remote_entries(
    ignore_hidden: bool,
    remote_dir: &Path,
    sftp: &Sftp
) -> Result<Vec<SshSyncEntry>, SyncError> {
    let remote_entries = sftp.readdir(remote_dir)?
        .iter()
        .filter_map(|(path, file_stat)| {
            let file_name = string_from_os_str(path.file_name().unwrap());

            let is_dir = file_stat.is_dir();
            let is_file = file_stat.is_file();

            if (is_dir || is_file) && !(ignore_hidden && file_name.starts_with('.')) {
                let modified = file_stat.mtime.unwrap();
                let path = path.clone();
                let size = if is_file {
                    file_stat.size.unwrap()
                } else {
                    0
                };

                let sync_entry = SshSyncEntry {
                    file_name,
                    is_dir,
                    modified,
                    path,
                    size
                };

                Some(sync_entry)
            } else {
                None
            }
        })
        .collect::<Vec<SshSyncEntry>>();

    Ok(remote_entries)
}

pub fn start_sync(
    context: &Arc<Context>,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    {
        let mut sync_state = context.sync_state.lock().unwrap();

        if sync_state.syncing() {
            let message = translations.already_deploying_can_only_deploy_again_when_current_deployment_is_finished;
            return Err(message.to_string());
        } else {
            sync_state.set_started();
        }
    }

    let (session, sftp) = connect(site, translations)?;

    let context = Arc::clone(context);
    let remote_path = PathBuf::from(site.ssh_config.path_unchecked());
    let translations = translations.clone();

    thread::spawn(move || {
        let mut log = |message: &str| {
            if simulate {
                let message = format!("(simulated) {message}");
                context.sync_state.lock().unwrap().log(&message)
            } else {
                context.sync_state.lock().unwrap().log(message)
            }
        };

        log(translations.beginning_deployment_over_ssh);

        let mut actions = Vec::new();

        match collect_sync_actions(
            &mut actions,
            true,
            &context.build_dir,
            &remote_path,
            &sftp
        ) {
            Ok(()) => {
                actions.sort_unstable_by(SshSyncAction::minimal_service_interruptions_sort);

                // TODO: Implement and make accessible through some option
                // actions.sort_unstable_by(SshSyncAction::minimal_storage_peaks_sort);

                log(translations.deployment_steps_successfully_determined_file_synchronization_will_now_commence);

                match execute_sync_actions(
                    &actions,
                    &mut log,
                    &sftp,
                    simulate,
                    &translations
                ) {
                    Ok(()) => log(translations.deployment_completed_successfully),
                    Err(err) => log(&err.to_string())
                }
            }
            Err(err) => log(&err.to_string())
        }

        let _ = session.disconnect(
            Some(DisconnectCode::ByApplication),
            "",
            None
        );

        context.sync_state.lock().unwrap().set_finished();
    });

    Ok(())
}

fn write_entry(
    local_entry: &SshSyncEntry,
    remote_path: &Path,
    sftp: &Sftp
) -> Result<(), SyncError> {
    let content = fs::read(&local_entry.path)?;

    sftp.create(&remote_path)?.write_all(&content)?;

    let file_stat = FileStat {
        atime: None,
        gid: None,
        mtime: Some(local_entry.modified),
        perm: None,
        size: None,
        uid: None
    };

    sftp.setstat(&remote_path, file_stat)?;

    Ok(())
}

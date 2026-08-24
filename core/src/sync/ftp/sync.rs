// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use chrono::{DateTime, Utc};
use suppaftp::{ImplFtpStream, Status, TlsStream};

use hyper8_translations::Translations;
use hyper8_util::string_from_os_string;

use crate::{
    Context,
    Site,
    SyncError,
    SyncState
};

use super::{FtpSyncAction, FtpSyncEntry};

/// Register `local_dir` and all recursively contained dirs and files for addition
fn collect_add_actions(
    actions: &mut Vec<FtpSyncAction>,
    local_dir: FtpSyncEntry,
    remote_containing_dir: &Path
) -> Result<(), SyncError> {
    // Add containing dir
    let remote_dir = remote_containing_dir.join(&local_dir.file_name);
    actions.push(FtpSyncAction::CreateDir { remote_path: remote_dir.clone() });

    let local_child_entries = get_local_entries(false, &local_dir.path)?;

    for local_entry in local_child_entries {
        if local_entry.is_dir {
            // Create child dir
            collect_add_actions(actions, local_entry, &remote_dir)?;
        } else {
            // Add child file
            let remote_path = remote_dir.join(&local_entry.file_name);
            actions.push(FtpSyncAction::AddFile { local_entry, remote_path });
        }
    }

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for deletion
fn collect_delete_actions<S: TlsStream>(
    actions: &mut Vec<FtpSyncAction>,
    impl_ftp_stream: &mut ImplFtpStream<S>,
    remote_dir: FtpSyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(false, impl_ftp_stream, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_delete_actions(actions, impl_ftp_stream, remote_entry)?;
        } else {
            // Delete child file
            actions.push(FtpSyncAction::DeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(FtpSyncAction::DeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for prerequisite deletion
fn collect_prerequisite_delete_actions<S: TlsStream>(
    actions: &mut Vec<FtpSyncAction>,
    impl_ftp_stream: &mut ImplFtpStream<S>,
    remote_dir: FtpSyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(false, impl_ftp_stream, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_prerequisite_delete_actions(actions, impl_ftp_stream, remote_entry)?;
        } else {
            // Delete child file
            actions.push(FtpSyncAction::PrereqDeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(FtpSyncAction::PrereqDeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Compare local and remote directories and files and register the respective
/// syncing actions (add, replace, recurse, remove, etc.) for each
fn collect_sync_actions<S: TlsStream>(
    actions: &mut Vec<FtpSyncAction>,
    // When we call this at the root/entry level of our sync operation we
    // ignore hidden files, as they might be owned by the server
    // infrastructure and should not be meddled with
    // (e.g. ".ftpquota" or ".htaccess" files)
    ignore_hidden: bool,
    impl_ftp_stream: &mut ImplFtpStream<S>,
    local_dir: &Path,
    remote_dir: &Path
) -> Result<(), SyncError> {
    let local_entries = get_local_entries(ignore_hidden, local_dir)?;
    let mut remote_entries = get_remote_entries(ignore_hidden, impl_ftp_stream, remote_dir)?;

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
                            impl_ftp_stream,
                            &local_entry.path,
                            &remote_entry.path
                        )?;
                    } else {
                        // Replace remote file with dir
                        actions.push(FtpSyncAction::PrereqDeleteFile { remote_entry });
                        collect_add_actions(actions, local_entry, remote_dir)?;
                    }
                } else if remote_entry.is_dir {
                    // Replace remote dir with file
                    collect_prerequisite_delete_actions(actions, impl_ftp_stream, remote_entry)?;

                    // Add new file to the remote
                    let remote_path = remote_dir.join(&local_entry.file_name);
                    actions.push(FtpSyncAction::AddFile { local_entry, remote_path });
                } else if remote_entry.modified != local_entry.modified {
                    let type_based_retardation_index = remote_entry.type_based_retardation_index();

                    // Replace remote file with new file
                    let action = FtpSyncAction::UpdateFile {
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
                    actions.push(FtpSyncAction::AddFile { local_entry, remote_path });
                }
            }
        };
    }

    // Register leftover remote entries that don't correspond to anything we
    // have locally for deletion, recursively for all child dirs too.
    for remote_entry in remote_entries {
        if remote_entry.is_dir {
            // Delete remote dir and all children, recursively
            collect_delete_actions(actions, impl_ftp_stream, remote_entry)?;
        } else {
            // Delete remote file
            let action = FtpSyncAction::DeleteFile { remote_entry };
            actions.push(action);
        }
    }

    Ok(())
}

/// The order of passed sync actions must be correct and sensible (!), i.e.
/// directories to be created that contain other directories need to be
/// created before their children, etc.
fn execute_sync_actions<S: TlsStream>(
    actions: &[FtpSyncAction],
    context: &Context,
    impl_ftp_stream: &mut ImplFtpStream<S>,
    simulate: bool
) -> Result<(), SyncError> {
    for (sync_action, step) in actions.iter().zip(1..) {
        context.log_sync_action_event(step, sync_action);

        if !simulate {
            match sync_action {
                FtpSyncAction::AddFile { local_entry, remote_path } => {
                    write_entry(impl_ftp_stream, local_entry, remote_path)?;
                }
                FtpSyncAction::CreateDir { remote_path } => {
                    impl_ftp_stream.mkdir(remote_path.to_string_lossy())?;
                }
                FtpSyncAction::DeleteDir { remote_path } |
                FtpSyncAction::PrereqDeleteDir { remote_path } => {
                    impl_ftp_stream.rmdir(remote_path.to_string_lossy())?;
                }
                FtpSyncAction::DeleteFile { remote_entry } |
                FtpSyncAction::PrereqDeleteFile { remote_entry } => {
                    impl_ftp_stream.rm(remote_entry.path.to_string_lossy())?;
                }
                FtpSyncAction::UpdateFile { local_entry, remote_entry, .. } => {
                    write_entry(impl_ftp_stream, local_entry, &remote_entry.path)?;
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
) -> Result<Vec<FtpSyncEntry>, SyncError> {
    let mut local_entries = Vec::new();

    for dir_entry_result in fs::read_dir(local_dir)? {
        let dir_entry = dir_entry_result?;
        let metadata = dir_entry.metadata()?;
        let is_dir = metadata.is_dir();

        if is_dir || metadata.is_file() {
            let file_name = string_from_os_string(dir_entry.file_name());

            if ignore_hidden && file_name.starts_with('.') { continue }

            let modified_system_time = metadata.modified().unwrap();
            let modified_date_time: DateTime<Utc> = modified_system_time.into();
            let modified = modified_date_time.format("%Y%m%d%H%M%S").to_string();
            let path = dir_entry.path();
            let size = if is_dir { 0 } else { metadata.len() };

            let sync_entry = FtpSyncEntry {
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
fn get_remote_entries<S: TlsStream>(
    ignore_hidden: bool,
    impl_ftp_stream: &mut ImplFtpStream<S>,
    remote_dir: &Path
) -> Result<Vec<FtpSyncEntry>, SyncError> {
    let remote_entries = impl_ftp_stream.mlsd(Some(&remote_dir.to_string_lossy()))?
        .iter()
        .filter_map(|item| {
            let (facts, file_name) = item.split_once(' ').unwrap();

            let is_dir = facts.contains("type=dir");
            let is_file = facts.contains("type=file");

            // The virtual directory entries "." and ".." have their own
            // types - "cdir" and "pdir" - and are therefore covered
            // (= excluded) by the is_dir || is_file check.
            if (is_dir || is_file) && !(ignore_hidden && file_name.starts_with('.')) {
                let file_name = file_name.to_string();
                let modified = facts
                    .split_once("modify=")
                    .unwrap()
                    .1
                    .split(';')
                    .next()
                    .unwrap()
                    .to_string();
                let path = remote_dir.join(&file_name);
                let size = if is_file {
                    facts
                        .split_once("size=")
                        .unwrap()
                        .1
                        .split(';')
                        .next()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap()
                } else {
                    0
                };

                let sync_entry = FtpSyncEntry {
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
        .collect::<Vec<FtpSyncEntry>>();

    Ok(remote_entries)
}

pub fn put_with_stream<S: TlsStream>(
    impl_ftp_stream: &mut ImplFtpStream<S>,
    local_path: &Path,
    remote_path: impl AsRef<Path>
) -> Result<(), SyncError> {
    let remote_path_str = remote_path.as_ref().to_string_lossy();

    // TODO: In the SSH sync implementation, BufReader based file transfer
    // has proven to be massively slower (than just doing file.write_all(...))
    // even for rather small files (e.g. 52MB). We should re-evaluate if we
    // can make these gains here too (but there might have been a reason to go
    // for the BufReader approach here, so check/test carefully).

    let mut data_stream = impl_ftp_stream.put_with_stream(remote_path_str)?;

    let file = File::open(local_path).unwrap();
    let mut reader = BufReader::new(file);

    loop {
        let buffer = reader.fill_buf().unwrap();

        if buffer.is_empty() { break; }

        let consumed_len = data_stream.write(buffer)?;
        reader.consume(consumed_len);
    }

    data_stream.flush()?;

    impl_ftp_stream.finalize_put_stream(data_stream)?;

    Ok(())
}

pub fn start_sync<S: Send + TlsStream + 'static>(
    connect: impl Fn(&str, &Site) -> Result<ImplFtpStream<S>, SyncError>,
    context: &Arc<Context>,
    prompted_password: Option<&str>,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    {
        let mut sync_state = context.sync_state.lock().unwrap();

        if let Some(sync_state) = &*sync_state && !sync_state.finished() {
            let message = translations.already_syncing_can_only_sync_again_when_current_sync_is_finished;
            return Err(message.to_string());
        }

        *sync_state = Some(SyncState::new(simulate));
    }

    let password = prompted_password
        .or(site.ftp_config.password.as_deref())
        .unwrap();

    let mut impl_ftp_stream = match connect(password, site) {
        Ok(impl_ftp_stream) => impl_ftp_stream,
        Err(err) => {
            let t_failed_to_connect = translations.failed_to_connect;
            let message = format!("{t_failed_to_connect}\n{err}");

            context.log_sync_error_and_finish(message.clone());

            return Err(message);
        }
    };

    let context = Arc::clone(context);
    let remote_path = PathBuf::from(site.ftp_config.path_unchecked());
    let translations = translations.clone();

    let secure = site.ftp_config.secure();

    thread::spawn(move || {
        let begin_message = if secure {
            translations.beginning_synchronization_over_secure_ftp_tls_ssl
        } else {
            translations.beginning_synchronization_over_insecure_ftp_unencrypted
        };

        context.log_generic_sync_event(begin_message);

        let mut actions = Vec::new();

        match collect_sync_actions(
            &mut actions,
            true,
            &mut impl_ftp_stream,
            &context.build_dir,
            &remote_path
        ) {
            Ok(()) => {
                context.log_sync_actions(actions.len() as u32, &translations);

                actions.sort_unstable_by(FtpSyncAction::minimal_service_interruptions_sort);

                // TODO: Implement and make accessible through some option
                // actions.sort_unstable_by(FtpSyncAction::minimal_storage_peaks_sort);

                match execute_sync_actions(
                    &actions,
                    &context,
                    &mut impl_ftp_stream,
                    simulate
                ) {
                    Ok(()) => context.log_generic_sync_event(translations.synchronization_completed_successfully),
                    Err(err) => context.log_sync_error_event(err.to_string())
                }
            }
            Err(err) => context.log_sync_error_event(err.to_string())
        }

        let _ = impl_ftp_stream.quit();

        context.sync_state.lock().unwrap().as_mut().unwrap().set_finished();
    });

    Ok(())
}

fn write_entry<S: TlsStream>(
    impl_ftp_stream: &mut ImplFtpStream<S>,
    local_entry: &FtpSyncEntry,
    remote_path: &Path
) -> Result<(), SyncError> {
    put_with_stream(impl_ftp_stream, &local_entry.path, remote_path)?;

    // https://en.wikipedia.org/wiki/File_Transfer_Protocol#Additional_commands
    // https://support.solarwinds.com/SuccessCenter/s/article/MFMT-FTP-command
    // https://forum.filezilla-project.org/viewtopic.php?t=15686
    let local_modified = &local_entry.modified;
    let remote_path = remote_path.display();
    let mfmt = format!("MFMT {local_modified} {remote_path}");

    let expected_codes = &[
        Status::CommandOk, // 200
        Status::File, // 212
        Status::Directory // 213
    ];

    impl_ftp_stream.custom_command(&mfmt, expected_codes)?;

    Ok(())
}

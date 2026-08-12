// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::UNIX_EPOCH;

use indoc::indoc;
use ssh2::{
    DisconnectCode,
    FileStat,
    Session,
    Sftp
};

use hyper8_translations::Translations;
use hyper8_util::{
    nanoid,
    string_from_os_str,
    string_from_os_string
};

use crate::{Context, Site, SitePath};

use super::{SyncAction, SyncEntry, SyncError};

/// SSH sync actions and entries represent modified time as 64-bit integers
/// (seconds since UNIX Epoch)
type SshSyncAction = SyncAction<u64>;
type SshSyncEntry = SyncEntry<u64>;

const DEFAULT_SSH_PORT: usize = 22;

const SSH_CONFIG_PROBE_FILE_NAME: &str = ".hyper8_ssh_config_probe.txt";
const SSH_CONFIG_PROBE_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://hyper8.org) to verify
    the configured server and path for ssh deployment matches up with the
    configured website url. It serves no further purpose anymore and may
    safely be deleted.
");

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

/// Uploads a file named ".hyper8_ssh_config_probe.txt", which also contains a random token,
/// to the remote, thereby first verifying that the deployment configuration is actually
/// correct and working. Then, right next, tries to fetch that same file through the public
/// url of the website (e.g. https://example.com/.hyper8_ssh_config_probe.txt), thereby confirming
/// that the website is set up correctly (although that is a side effect), but more importantly
/// ensuring that no other website (or remote content of any kind) is accidentally wiped if the
/// remote path has been accidentally set incorrectly. Regardless of failure or success of the
/// check, the probe file is removed from the remote again, leaving a clean state.
pub fn config_probe(
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    let probe_signature = nanoid();
    let probe_content = format!("{SSH_CONFIG_PROBE_TEXT}\n\n{probe_signature}");

    let port = site.ssh_config.port.unwrap_or(DEFAULT_SSH_PORT);
    let server = site.ssh_config.server_unchecked();

    let address = format!("{server}:{port}");

    let tcp = TcpStream::connect(address).unwrap();

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    let user = site.ssh_config.user_unchecked();
    // TODO: Maybe dynamically fallback to agent/keyfile-based
    // authentication if there is no password (none is required for the SSH
    // configuration to be complete right now)
    let password = site.ssh_config.password_unchecked();

    session.userauth_password(user, password).unwrap();

    if !session.authenticated() {
        return Err(String::from("Not authenticated in spite of successful userauth_password invocation"));
    }

    let sftp = match session.sftp() {
        Ok(sftp) => sftp,
        Err(err) => return Err(format!("SFTP subsystem could not be initialized ({err})"))
    };

    let remote_dir = site.ssh_config.path_unchecked();

    let remote_probe_path = Path::new(remote_dir).join(SSH_CONFIG_PROBE_FILE_NAME);

    // Write probe file
    match sftp.create(&remote_probe_path) {
        Ok(mut file) => {
            match file.write_all(probe_content.as_bytes()) {
                Ok(_) => (),
                Err(err) => return Err(format!("Could not write probe file ({err})"))
            }
        }
        Err(err) => return Err(format!("Could not write probe file ({err})"))
    }

    let public_probe_url = site.absolute_url_file_unchecked(SSH_CONFIG_PROBE_FILE_NAME, &SitePath::root_path());

    let remove_probe_and_return = |previous_result: Result<(), String>| -> Result<(), String> {
        match sftp.unlink(&remote_probe_path) {
            Ok(()) => {
                let _ = session.disconnect(
                    Some(DisconnectCode::ByApplication),
                    "",
                    None
                );

                previous_result
            }
            Err(file_removal_err) => {
                // If removing the probe fails when we already had a previous
                // error, we prioritize returning the earlier error, as it
                // likely provides a more meaningful context.
                if previous_result.is_err() {
                    previous_result
                } else {
                    let message = translations.the_test_file_could_not_be_removed_again(
                        &file_removal_err.to_string()
                    );
                    Err(message)
                }
            }
        }
    };

    // Fetch probe file through public url and return result
    match minreq::get(public_probe_url.clone()).send() {
        Ok(response) => {
            match response.as_str() {
                Ok(body) => {
                    if body == probe_content {
                        remove_probe_and_return(Ok(()))
                    } else {
                        let truncate = |text: &str| -> String {
                            match text.char_indices().nth(64) {
                                Some((index, _)) => format!("{}…", &text[..index]),
                                None => text.to_string()
                            }
                        };

                        let err = translations.the_test_file_contains_xxx_but_the_response_contained_xxx(
                            &truncate(&probe_content),
                            &truncate(body)
                        );

                        let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                            &err,
                            &public_probe_url,
                            &remote_probe_path.to_string_lossy()
                        );
                        remove_probe_and_return(Err(message))
                    }
                }
                Err(err) => {
                    let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                        &err.to_string(),
                        &public_probe_url,
                        &remote_probe_path.to_string_lossy()
                    );
                    remove_probe_and_return(Err(message))
                }
            }
        }
        Err(err) => {
            let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                &err.to_string(),
                &public_probe_url,
                &remote_probe_path.to_string_lossy()
            );
            remove_probe_and_return(Err(message))
        }
    }
}

/// The order of passed sync actions must be correct and sensible (!), i.e.
/// directories to be created that contain other directories need to be
/// created before their children, etc.
fn execute_sync_actions(
    actions: &[SshSyncAction],
    report: &mut impl FnMut(&str),
    sftp: &Sftp,
    translations: &Translations
) -> Result<(), SyncError> {
    for action in actions {
        match action {
            SshSyncAction::AddFile { local_entry, remote_path } => {
                let message = translations.adding_file_xxx(&remote_path.to_string_lossy());
                report(&message);

                write_entry(local_entry, remote_path, sftp)?;
            }
            SshSyncAction::CreateDir { remote_path } => {
                let message = translations.creating_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                // 775 (octal) = rwxr-xr-x (7 = rwx (owner), 5 = rx (group/others))
                let mode = 0o755;

                sftp.mkdir(remote_path, mode)?;
            }
            SshSyncAction::DeleteDir { remote_path } |
            SshSyncAction::PrereqDeleteDir { remote_path } => {
                let message = translations.deleting_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                sftp.rmdir(remote_path)?;
            }
            SshSyncAction::DeleteFile { remote_entry } |
            SshSyncAction::PrereqDeleteFile { remote_entry } => {
                let message = translations.deleting_file_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                sftp.unlink(&remote_entry.path)?;
            }
            SshSyncAction::UpdateFile { local_entry, remote_entry, .. } => {
                let message = translations.updating_file_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                write_entry(local_entry, &remote_entry.path, sftp)?;
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

pub fn start_deploy(
    context: &Arc<Context>,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    {
        let mut deployment = context.deploying.lock().unwrap();
        if deployment.active() {
            let message = translations.already_deploying_can_only_deploy_again_when_current_deployment_is_finished;
            return Err(message.to_string());
        } else {
            deployment.start();
        }
    }

    let port = site.ssh_config.port.unwrap_or(DEFAULT_SSH_PORT);
    let server = site.ssh_config.server_unchecked();

    let address = format!("{server}:{port}");

    let tcp = TcpStream::connect(address).unwrap();

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    let user = site.ssh_config.user_unchecked();
    // TODO: Maybe dynamically fallback to agent/keyfile-based
    // authentication if there is no password (none is required for the SSH
    // configuration to be complete right now)
    let password = site.ssh_config.password_unchecked();

    session.userauth_password(user, password).unwrap();

    if !session.authenticated() {
        return Err(String::from("Not authenticated in spite of successful userauth_password invocation"));
    }

    let sftp = match session.sftp() {
        Ok(sftp) => sftp,
        Err(err) => return Err(format!("SFTP subsystem could not be initialized ({err})"))
    };

    let context = Arc::clone(context);
    let remote_path = PathBuf::from(site.ssh_config.path_unchecked());
    let translations = translations.clone();

    thread::spawn(move || {
        let mut report = |message: &str| {
            context.deploying.lock().unwrap().report(message);
        };

        report(translations.beginning_deployment_over_ssh);

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

                report(translations.deployment_steps_successfully_determined_file_synchronization_will_now_commence);

                match execute_sync_actions(
                    &actions,
                    &mut report,
                    &sftp,
                    &translations
                ) {
                    Ok(()) => report(translations.deployment_completed_successfully),
                    Err(err) => report(&err.to_string())
                }
            }
            Err(err) => report(&err.to_string())
        }

        let _ = session.disconnect(
            Some(DisconnectCode::ByApplication),
            "",
            None
        );

        context.deploying.lock().unwrap().complete();
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

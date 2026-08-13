// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use chrono::{DateTime, Utc};
use indoc::indoc;

use hyper8_translations::Translations;
use hyper8_util::{nanoid, string_from_os_string};

use crate::{Context, Site, SitePath};

use super::{
    FtpStreamWrapper,
    SyncAction,
    SyncEntry,
    SyncError
};

/// FTP sync actions and entries represent modified time as date strings of
/// the form yyyymmddHHMMSS, e.g. "20250509105623"
type FtpSyncAction = SyncAction<String>;
type FtpSyncEntry = SyncEntry<String>;

const FTP_DEPLOYMENT_VERIFICATION_FILE_NAME: &str = ".hyper8_ftp_deployment_verification.txt";
const FTP_DEPLOYMENT_VERIFICATION_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://hyper8.org) to verify
    the configured server and path for FTP deployment matches up with the
    configured website url. It serves no further purpose anymore and may
    safely be deleted.
");

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
fn collect_delete_actions(
    actions: &mut Vec<FtpSyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    remote_dir: FtpSyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(ftp_stream_wrapper, false, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;
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
fn collect_prerequisite_delete_actions(
    actions: &mut Vec<FtpSyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    remote_dir: FtpSyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(ftp_stream_wrapper, false, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_prerequisite_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;
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
fn collect_sync_actions(
    actions: &mut Vec<FtpSyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    // When we call this at the root/entry level of our sync operation we
    // ignore hidden files, as they might be owned by the server
    // infrastructure and should not be meddled with
    // (e.g. ".ftpquota" or ".htaccess" files)
    ignore_hidden: bool,
    local_dir: &Path,
    remote_dir: &Path
) -> Result<(), SyncError> {
    let local_entries = get_local_entries(ignore_hidden, local_dir)?;
    let mut remote_entries = get_remote_entries(ftp_stream_wrapper, ignore_hidden, remote_dir)?;

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
                            ftp_stream_wrapper,
                            false,
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
                    collect_prerequisite_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;

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
            collect_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;
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
fn execute_sync_actions(
    actions: &[FtpSyncAction],
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    report: &mut impl FnMut(&str),
    translations: &Translations
) -> Result<(), SyncError> {
    for action in actions {
        match action {
            FtpSyncAction::AddFile { local_entry, remote_path } => {
                let message = translations.adding_file_xxx(&remote_path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.put_with_stream(&local_entry.path, remote_path)?;

                // https://en.wikipedia.org/wiki/File_Transfer_Protocol#Additional_commands
                // https://support.solarwinds.com/SuccessCenter/s/article/MFMT-FTP-command
                // https://forum.filezilla-project.org/viewtopic.php?t=15686
                let local_modified = &local_entry.modified;
                let remote_path = remote_path.display();
                let mfmt = format!("MFMT {local_modified} {remote_path}");

                ftp_stream_wrapper.custom_command(&mfmt)?;
            }
            FtpSyncAction::CreateDir { remote_path } => {
                let message = translations.creating_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.mkdir(remote_path)?;
            }
            FtpSyncAction::DeleteDir { remote_path } |
            FtpSyncAction::PrereqDeleteDir { remote_path } => {
                let message = translations.deleting_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.rmdir(remote_path)?;
            }
            FtpSyncAction::DeleteFile { remote_entry } |
            FtpSyncAction::PrereqDeleteFile { remote_entry } => {
                let message = translations.deleting_file_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.rm(&remote_entry.path)?;
            }
            FtpSyncAction::UpdateFile { local_entry, remote_entry, .. } => {
                let message = translations.updating_file_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.put_with_stream(&local_entry.path, &remote_entry.path)?;

                // https://en.wikipedia.org/wiki/File_Transfer_Protocol#Additional_commands
                // https://support.solarwinds.com/SuccessCenter/s/article/MFMT-FTP-command
                // https://forum.filezilla-project.org/viewtopic.php?t=15686
                let local_modified = &local_entry.modified;
                let remote_path = remote_entry.path.display();
                let mfmt = format!("MFMT {local_modified} {remote_path}");

                ftp_stream_wrapper.custom_command(&mfmt)?;
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
fn get_remote_entries(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    ignore_hidden: bool,
    remote_dir: &Path
) -> Result<Vec<FtpSyncEntry>, SyncError> {
    let remote_entries = ftp_stream_wrapper.mlsd(remote_dir)?
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

pub fn start_ftp_deployment(
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

    let mut ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;

    let context = Arc::clone(context);
    let remote_path = PathBuf::from(site.ftp_config.path_unchecked());
    let translations = translations.clone();

    thread::spawn(move || {
        let mut report = |message: &str| {
            context.deploying.lock().unwrap().report(message);
        };

        if ftp_stream_wrapper.is_secure() {
            report(translations.beginning_deployment_over_secure_ftp_tls_ssl);
        } else {
            report(translations.beginning_deployment_over_insecure_ftp_unencrypted);
        }

        let mut actions = Vec::new();

        match collect_sync_actions(
            &mut actions,
            &mut ftp_stream_wrapper,
            true,
            &context.build_dir,
            &remote_path
        ) {
            Ok(()) => {
                actions.sort_unstable_by(FtpSyncAction::minimal_service_interruptions_sort);

                // TODO: Implement and make accessible through some option
                // actions.sort_unstable_by(FtpSyncAction::minimal_storage_peaks_sort);

                report(translations.deployment_steps_successfully_determined_file_synchronization_will_now_commence);

                match execute_sync_actions(
                    &actions,
                    &mut ftp_stream_wrapper,
                    &mut report,
                    &translations
                ) {
                    Ok(()) => report(translations.deployment_completed_successfully),
                    Err(err) => report(&err.to_string())
                }
            }
            Err(err) => report(&err.to_string())
        }

        let _ = ftp_stream_wrapper.quit();

        context.deploying.lock().unwrap().complete();
    });

    Ok(())
}

/// Uploads a file named ".hyper8_ftp_deployment_verification.txt", which also contains a
/// random token, to the remote, thereby first verifying that the deployment
/// configuration is correct and working. Then, right next, tries to fetch
/// that same file through the public url of the website
/// (e.g. https://example.com/.hyper8_ftp_deployment_verification.txt), thereby confirming
/// that the website is set up correctly and, more importantly, that no other
/// website (or remote content of any kind) will be wiped due to a wrongly
/// set remote path. Regardless of failure or success of the check, the probe
/// file is removed from the remote again, leaving a clean state.
pub fn verify_ftp_deployment_config(
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    let mut ftp_stream_wrapper = FtpStreamWrapper::connect(site)?;

    let probe_signature = nanoid();
    let probe_content = format!("{FTP_DEPLOYMENT_VERIFICATION_TEXT}\n\n{probe_signature}");

    let remote_dir = site.ftp_config.path_unchecked();

    // Check if the configured remote directory exists (by changing into it)
    if let Err(remote_dir_err) = ftp_stream_wrapper.cwd(&remote_dir) {
        // If the configured remote directory does not exist, try changing
        // into each of its ancestor directories (going from deepest to
        // shallowest) in order to determine and report the first existing
        // directory that is part of the user's configured remote directory
        // path.
        while let Some(ancestor_dir) = Path::new(remote_dir).parent() {
            if ftp_stream_wrapper.cwd(&ancestor_dir).is_ok() {
                let message = translations.remote_dir_does_not_exist_ancestor_dir_found(
                    &ancestor_dir.to_string_lossy(),
                    remote_dir,
                    &remote_dir_err.to_string()
                );

                return Err(message);
            }
        }

        let message = translations.neither_remote_dir_xxx_nor_any_ancestor_dir_found(
            remote_dir,
            &remote_dir_err.to_string()
        );

        return Err(message);
    }

    let remote_probe_path = Path::new(remote_dir).join(FTP_DEPLOYMENT_VERIFICATION_FILE_NAME);

    // Write probe file
    let mut reader = Cursor::new(probe_content.as_bytes());
    if let Err(err) = ftp_stream_wrapper.put_file(&remote_probe_path, &mut reader) {
        let t_message = translations.the_test_file_upload_for_verifying_your_deployment_configuration_did_not_succeed;
        let message = format!("{t_message} ({err})");
        return Err(message);
    }

    let public_probe_url = site.absolute_url_file_unchecked(FTP_DEPLOYMENT_VERIFICATION_FILE_NAME, &SitePath::root_path());

    let mut remove_probe_and_return = |previous_result: Result<(), String>| -> Result<(), String> {
        match ftp_stream_wrapper.rm(&remote_probe_path) {
            Ok(()) => {
                let _ = ftp_stream_wrapper.quit();
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
                        &remote_probe_path.to_string_lossy(),
                        &public_probe_url
                    );
                    remove_probe_and_return(Err(message))
                }
            }
        }
        Err(err) => {
            let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                &err.to_string(),
                &remote_probe_path.to_string_lossy(),
                &public_probe_url
            );
            remove_probe_and_return(Err(message))
        }
    }
}

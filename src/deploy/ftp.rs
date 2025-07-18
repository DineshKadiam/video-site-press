// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use chrono::{DateTime, Utc};
use indoc::indoc;
use log::trace;
use suppaftp::{
    FtpError,
    FtpResult,
    FtpStream,
    NativeTlsConnector,
    NativeTlsFtpStream,
    Status
};
use suppaftp::native_tls::TlsConnector;
use suppaftp::types::FileType;

use crate::{Context, Site, SitePath, Translations};
use crate::util::{
    nanoid,
    string_from_os_string
};

const DEFAULT_PORT: usize = 21;

const FTP_CONFIG_PROBE_FILE_NAME: &str = ".hyper8_ftp_config_probe.txt";
const FTP_CONFIG_PROBE_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://simonrepp.com/hyper8) to verify
    the configured server and path for FTP deployment matches up with the
    configured website url. It serves no further purpose anymore and may
    safely be deleted.
");

const EARLIER: Ordering = Ordering::Less;
const LATER: Ordering = Ordering::Greater;

#[derive(Clone, Debug)]
pub struct FtpConfig {
    pub password: Option<String>,
    pub path: Option<String>,
    pub port: Option<usize>,
    // Whether to use FTP or FTPS
    pub secure: Option<bool>,
    pub server: Option<String>,
    pub user: Option<String>
}

/// The crate we use for FTP interaction (suppaftp) does not publicly expose
/// the traits we would need to generically implement insecure/secure FTP
/// stream interaction, therefore we put both stream types into our own
/// wrapper and reimplement a generic function call for each FTP command we
/// use from the crate.
pub enum FtpStreamWrapper {
    Insecure(FtpStream),
    Secure(NativeTlsFtpStream)
}

enum SyncAction {
    AddFile {
        local_entry: SyncEntry,
        remote_path: PathBuf
    },
    CreateDir { remote_path: PathBuf },
    DeleteDir { remote_path: PathBuf },
    DeleteFile { remote_entry: SyncEntry },
    PrereqDeleteDir { remote_path: PathBuf },
    PrereqDeleteFile { remote_entry: SyncEntry },
    UpdateFile {
        local_entry: SyncEntry,
        remote_entry: SyncEntry,
        /// During deployment we want some files to be updated later, or even
        /// at the latest possible point. For instance, updating the RSS feed
        /// files should only happen as the very last step, as everything
        /// that the feed points to should be uploaded and in order the
        /// moment that feed subscribers receive an update! This index is
        /// derived from the file extensions and helps us to order files
        /// accordingly.
        type_based_retardation_index: usize
    }
}

#[derive(Debug)]
struct SyncEntry {
    pub file_name: String,
    pub is_dir: bool,
    // yyyymmddHHMMSS (e.g. "20250509105623")
    pub modified: String,
    pub path: PathBuf,
    pub size: usize
}

pub struct SyncError(String);

impl FtpConfig {
    pub fn complete(&self) -> bool {
        self.password.is_some() &&
        self.path.is_some() &&
        self.server.is_some() &&
        self.user.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.password.is_none() &&
        self.path.is_none() &&
        self.port.is_none() &&
        self.secure.is_none() &&
        self.server.is_none() &&
        self.user.is_none()
    }

    pub fn new() -> FtpConfig {
        FtpConfig {
            password: None,
            path: None,
            port: None,
            secure: None,
            server: None,
            user: None
        }
    }

    pub fn password_unchecked(&self) -> &str {
        self.password.as_ref().unwrap()
    }

    pub fn path_unchecked(&self) -> &str {
        self.path.as_ref().unwrap()
    }

    pub fn secure(&self) -> bool {
        self.secure.is_none() || self.secure.is_some_and(|secure| secure)
    }

    pub fn server_unchecked(&self) -> &str {
        self.server.as_ref().unwrap()
    }

    pub fn user_unchecked(&self) -> &str {
        self.user.as_ref().unwrap()
    }
}

impl FtpStreamWrapper {
    pub fn connect(site: &Site) -> Result<FtpStreamWrapper, SyncError> {
        let password = site.ftp_config.password_unchecked();
        let port = site.ftp_config.port.unwrap_or(DEFAULT_PORT);
        let server = site.ftp_config.server_unchecked();
        let user = site.ftp_config.user_unchecked();

        let server_and_port = format!("{server}:{port}");

        let ftp_stream_wrapper = if site.ftp_config.secure() {
            let stream = NativeTlsFtpStream::connect(server_and_port)?;

            let tls_connector = TlsConnector::new()
                .unwrap();

            // Uncomment and use as alternative for testing against ad-hoc
            // server setups with invalid certificates.
            // let tls_connector = TlsConnector::builder()
            //     .danger_accept_invalid_certs(true)
            //     .build()
            //     .unwrap();

            let mut stream_secure = stream.into_secure(NativeTlsConnector::from(tls_connector), server)?;

            stream_secure.login(user, password)?;
            stream_secure.transfer_type(FileType::Binary)?;

            FtpStreamWrapper::Secure(stream_secure)
        } else {
            let mut stream = FtpStream::connect(server_and_port).unwrap();

            stream.login(user, password)?;
            stream.transfer_type(FileType::Binary)?;

            FtpStreamWrapper::Insecure(stream)
        };

        Ok(ftp_stream_wrapper)
    }

    pub fn custom_command(&mut self, command: &str) -> FtpResult<()> {
        let expected_codes = &[
            Status::CommandOk, // 200
            Status::File, // 212
            Status::Directory // 213
        ];

        trace!("custom_command {command}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.custom_command(command, expected_codes).map(|_response| ()),
            FtpStreamWrapper::Secure(stream) => stream.custom_command(command, expected_codes).map(|_response| ())
        }
    }

    pub fn is_secure(&self) -> bool{
        match self {
            FtpStreamWrapper::Insecure(_) => false,
            FtpStreamWrapper::Secure(_) => true
        }
    }

    pub fn mlsd(&mut self, path: impl AsRef<Path>) -> FtpResult<Vec<String>> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("mlsd {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.mlsd(Some(&path_str)),
            FtpStreamWrapper::Secure(stream) => stream.mlsd(Some(&path_str))
        }
    }

    pub fn mkdir(&mut self, path: impl AsRef<Path>) -> FtpResult<()> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("mkdir {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.mkdir(path_str),
            FtpStreamWrapper::Secure(stream) => stream.mkdir(path_str)
        }
    }

    pub fn put_file(&mut self, path: impl AsRef<Path>, mut reader: impl Read) -> FtpResult<u64> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("put_file {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.put_file(path_str, &mut reader),
            FtpStreamWrapper::Secure(stream) => stream.put_file(path_str, &mut reader)
        }
    }

    pub fn put_with_stream(
        &mut self,
        local_path: &Path,
        remote_path: impl AsRef<Path>
    ) -> Result<(), SyncError> {
        let remote_path_str = remote_path.as_ref().to_string_lossy();

        trace!("put_with_stream {} {remote_path_str}", local_path.display());

        match self {
            FtpStreamWrapper::Insecure(stream) => {
                let mut data_stream = stream.put_with_stream(remote_path_str)?;

                let file = File::open(local_path).unwrap();
                let mut reader = BufReader::new(file);

                loop {
                    let buffer = reader.fill_buf().unwrap();

                    if buffer.is_empty() { break; }

                    let consumed_len = data_stream.write(buffer)?;
                    reader.consume(consumed_len);
                }

                data_stream.flush()?;

                stream.finalize_put_stream(data_stream)?;

                Ok(())
            }
            FtpStreamWrapper::Secure(stream) => {
                let mut data_stream = stream.put_with_stream(remote_path_str)?;

                let file = File::open(local_path).unwrap();
                let mut reader = BufReader::new(file);

                loop {
                    let buffer = reader.fill_buf().unwrap();

                    if buffer.is_empty() { break; }

                    let consumed_len = data_stream.write(buffer)?;
                    reader.consume(consumed_len);
                }

                data_stream.flush()?;

                stream.finalize_put_stream(data_stream)?;

                Ok(())
            }
        }
    }

    pub fn quit(&mut self) -> FtpResult<()> {
        match self {
            FtpStreamWrapper::Insecure(stream) => stream.quit(),
            FtpStreamWrapper::Secure(stream) => stream.quit()
        }
    }

    pub fn rm(&mut self, path: impl AsRef<Path>) -> FtpResult<()> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("rm {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.rm(path_str),
            FtpStreamWrapper::Secure(stream) => stream.rm(path_str)
        }
    }

    pub fn rmdir(&mut self, path: impl AsRef<Path>) -> FtpResult<()> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("rmdir {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.rmdir(path_str),
            FtpStreamWrapper::Secure(stream) => stream.rmdir(path_str)
        }
    }
}

impl Debug for SyncAction {
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

impl SyncEntry {
    pub fn type_based_retardation_index(&self) -> usize {
        match Path::new(&self.file_name).extension() {
            Some(extension) => {
                match &*extension.to_ascii_lowercase().to_string_lossy() {
                    "atom" | "rss" => 5,
                    "html" => 4,
                    "css" => 3,
                    "js" => 2,
                    "avif" | "jpeg" | "jpg" | "png" | "webp" => 1,
                    _ => 0
                }
            }
            None => 0
        }
    }
}

impl From<FtpError> for SyncError {
    fn from(error: FtpError) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> SyncError {
        SyncError(error.to_string())
    }
}

impl From<SyncError> for String {
    fn from(sync_error: SyncError) -> String {
        sync_error.0
    }
}

impl Display for SyncError {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Register `local_dir` and all recursively contained dirs and files for addition
fn collect_add_actions(
    actions: &mut Vec<SyncAction>,
    local_dir: SyncEntry,
    remote_containing_dir: &Path
) -> Result<(), SyncError> {
    // Add containing dir
    let remote_dir = remote_containing_dir.join(&local_dir.file_name);
    actions.push(SyncAction::CreateDir { remote_path: remote_dir.clone() });

    let local_child_entries = get_local_entries(false, &local_dir.path)?;

    for local_entry in local_child_entries {
        if local_entry.is_dir {
            // Create child dir
            collect_add_actions(actions, local_entry, &remote_dir)?;
        } else {
            // Add child file
            let remote_path = remote_dir.join(&local_entry.file_name);
            actions.push(SyncAction::AddFile { local_entry, remote_path });
        }
    }

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for deletion
fn collect_delete_actions(
    actions: &mut Vec<SyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    remote_dir: SyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(ftp_stream_wrapper, false, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;
        } else {
            // Delete child file
            actions.push(SyncAction::DeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(SyncAction::DeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Register `remote_dir` and all recursively contained dirs and files for prerequisite deletion
fn collect_prerequisite_delete_actions(
    actions: &mut Vec<SyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    remote_dir: SyncEntry
) -> Result<(), SyncError> {
    let remote_child_entries = get_remote_entries(ftp_stream_wrapper, false, &remote_dir.path)?;

    for remote_entry in remote_child_entries {
        if remote_entry.is_dir {
            // Delete child dir
            collect_prerequisite_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;
        } else {
            // Delete child file
            actions.push(SyncAction::PrereqDeleteFile { remote_entry });
        }
    }

    // Delete containing dir
    actions.push(SyncAction::PrereqDeleteDir { remote_path: remote_dir.path });

    Ok(())
}

/// Compare local and remote directories and files within this directory and
/// register the respective syncing actions (add, replace, recurse, remove,
/// etc.) for each.
fn collect_sync_actions(
    actions: &mut Vec<SyncAction>,
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    local_dir: &Path,
    remote_dir: &Path,
    // When we call this at the root/entry level of our sync operation we
    // ignore hidden files, as they might be owned by the server
    // infrastructure and should not be meddled with
    // (e.g. ".ftpquota" or ".htaccess" files)
    ignore_hidden: bool
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
                            &local_entry.path,
                            &remote_entry.path,
                            false
                        )?;
                    } else {
                        // Replace remote file with dir
                        actions.push(SyncAction::PrereqDeleteFile { remote_entry });
                        collect_add_actions(actions, local_entry, remote_dir)?;
                    }
                } else if remote_entry.is_dir {
                    // Replace remote dir with file
                    collect_prerequisite_delete_actions(actions, ftp_stream_wrapper, remote_entry)?;

                    // Add new file to the remote
                    let remote_path = remote_dir.join(&local_entry.file_name);
                    actions.push(SyncAction::AddFile { local_entry, remote_path });
                } else if remote_entry.modified != local_entry.modified {
                    let type_based_retardation_index = remote_entry.type_based_retardation_index();

                    // Replace remote file with new file
                    let action = SyncAction::UpdateFile {
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
                    actions.push(SyncAction::AddFile { local_entry, remote_path });
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
            let action = SyncAction::DeleteFile { remote_entry };
            actions.push(action);
        }
    }

    Ok(())
}

/// Uploads a file named ".hyper8_ftp_config_probe.txt", which also contains a
/// random token, to the remote, thereby first verifying that the deployment
/// configuration is correct and working. Then, right next, tries to fetch
/// that same file through the public url of the website
/// (e.g. https://example.com/.hyper8_config_probe.txt), thereby confirming
/// that the website is set up correctly and, more importantly, that no other
/// website (or remote content of any kind) will be wiped due to a wrongly
/// set remote path. Regardless of failure or success of the check, the probe
/// file is removed from the remote again, leaving a clean state.
pub fn config_probe(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    let probe_signature = nanoid();
    let probe_content = format!("{FTP_CONFIG_PROBE_TEXT}\n\n{probe_signature}");
    let remote_probe_path = Path::new(site.ftp_config.path_unchecked()).join(FTP_CONFIG_PROBE_FILE_NAME);

    // Write probe file
    let mut reader = Cursor::new(probe_content.as_bytes());
    if let Err(err) = ftp_stream_wrapper.put_file(&remote_probe_path, &mut reader) {
        return Err(err.to_string());
    }

    let public_probe_url = site.absolute_url_file_unchecked(FTP_CONFIG_PROBE_FILE_NAME, &SitePath::root_path());

    let mut remove_probe_and_return = |previous_result: Result<(), String>| -> Result<(), String> {
        match ftp_stream_wrapper.rm(&remote_probe_path) {
            Ok(()) => previous_result,
            Err(file_removal_err) => {
                // If removing the probe fails when we already had a previous
                // error, we prioritize returning the earlier error, as it
                // likely provides a more meaningful context.
                if previous_result.is_err() {
                    previous_result
                } else {
                    Err(file_removal_err.to_string())
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
                            match text.char_indices().nth(10) {
                                Some((index, _)) => format!("{}…", &text[..index]),
                                None => text.to_string()
                            }
                        };

                        // TODO: Translate
                        let err = format!("The test file contains '{}' but the response contained '{}'", truncate(&probe_content), truncate(body));
                        let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                            &remote_probe_path.to_string_lossy(),
                            &public_probe_url,
                            &err
                        );
                        remove_probe_and_return(Err(message))
                    }
                }
                Err(err) => {
                    let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                        &remote_probe_path.to_string_lossy(),
                        &public_probe_url,
                        &err.to_string()
                    );
                    remove_probe_and_return(Err(message))
                }
            }
        }
        Err(err) => {
            let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                &remote_probe_path.to_string_lossy(),
                &public_probe_url,
                &err.to_string()
            );
            remove_probe_and_return(Err(message))
        }
    }
}

/// The order of passed sync actions must be correct and sensible (!), i.e.
/// directories to be created that contain other directories need to be
/// created before their children, etc.
fn execute_sync_actions(
    actions: &[SyncAction],
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    report: &mut impl FnMut(&str),
    translations: &Translations
) -> Result<(), SyncError> {
    for action in actions {
        match action {
            SyncAction::AddFile { local_entry, remote_path } => {
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
            SyncAction::CreateDir { remote_path } => {
                let message = translations.creating_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.mkdir(&remote_path)?;
            }
            SyncAction::DeleteDir { remote_path } |
            SyncAction::PrereqDeleteDir { remote_path } => {
                let message = translations.deleting_directory_xxx(&remote_path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.rmdir(&remote_path)?;
            }
            SyncAction::DeleteFile { remote_entry } |
            SyncAction::PrereqDeleteFile { remote_entry } => {
                let message = translations.deleting_file_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                ftp_stream_wrapper.rm(&remote_entry.path)?;
            }
            SyncAction::UpdateFile { local_entry, remote_entry, .. } => {
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
) -> Result<Vec<SyncEntry>, SyncError> {
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
            let size = if is_dir { 0 } else { metadata.len() as usize };

            let sync_entry = SyncEntry {
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
/// from a remote dir. `remote_dir` must be an absolute path (in the remote
/// context).
fn get_remote_entries(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    ignore_hidden: bool,
    remote_dir: &Path
) -> Result<Vec<SyncEntry>, SyncError> {
    let remote_entries = ftp_stream_wrapper.mlsd(remote_dir)?
        .iter()
        .filter_map(|item| {
            let (facts, file_name) = item.split_once(' ').unwrap();

            let is_dir = facts.contains("type=dir");
            let is_file = facts.contains("type=file");

            // The virtual directory entries "." and ".." have their own
            // types - "cdir" and "pdir" - and are therefore covered
            // (= excluded) by the is_dir || is_file check.
            if is_dir || is_file &&
                !(ignore_hidden && file_name.starts_with('.')) {
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
                        .parse::<usize>()
                        .unwrap()
                } else {
                    0
                };

                let sync_entry = SyncEntry {
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
        .collect::<Vec<SyncEntry>>();

    Ok(remote_entries)
}

// Default strategy (prioritize avoidance of glitches/service interruptions, at cost of temporary storage usage peaks)
// "Prioritize avoidance of service interruptions during deployment"
//
// 1. Delete all files that need to be deleted as a prerequisite to later actions, going from biggest to smallest
// 2. Delete all directories that need to be deleted as a prerequisite to later actions, going from deepest to shallowest
// 3. Create all missing directories, going from shallowest to deepest
// 4. Upload all completely new files, going from biggest to smallest
// 5. Replace updated files, going from biggest to smallest
// 6. Delete all files, from biggest to smallest
// 7. Delete all directories, going from deepest to shallowest
fn order_sync_actions_quality_approach(actions: &mut Vec<SyncAction>) {
    actions.sort_unstable_by(|a, b| {
        match a {
            SyncAction::AddFile { local_entry: a_local_entry, .. } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { .. } => LATER,
                    SyncAction::CreateDir { .. } => LATER,
                    SyncAction::AddFile { local_entry: b_local_entry, .. } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_local_entry.size.cmp(&a_local_entry.size)
                    }
                    SyncAction::UpdateFile { .. } => EARLIER,
                    SyncAction::DeleteFile { .. } => EARLIER,
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
            SyncAction::CreateDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { .. } => LATER,
                    SyncAction::CreateDir { remote_path: b_remote_path } => {
                        // Shallower directories are created earlier, deeper directories later
                        // (Create /foo -> Create /foo/bar -> Create /foo/bar/baz)
                        a_remote_path.components().count().cmp(&b_remote_path.components().count())
                    }
                    SyncAction::AddFile { .. } => EARLIER,
                    SyncAction::UpdateFile { .. } => EARLIER,
                    SyncAction::DeleteFile { .. } => EARLIER,
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
            SyncAction::DeleteDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { .. } => LATER,
                    SyncAction::CreateDir { .. } => LATER,
                    SyncAction::AddFile { .. } => LATER,
                    SyncAction::UpdateFile { .. } => LATER,
                    SyncAction::DeleteFile { .. } => LATER,
                    SyncAction::DeleteDir { remote_path: b_remote_path } => {
                        // Deeper directories are deleted earlier, shallower directories later
                        // (Delete /foo/bar/baz -> Delete /foo/bar -> Delete /foo)
                        b_remote_path.components().count().cmp(&a_remote_path.components().count())
                    }
                }
            }
            SyncAction::DeleteFile { remote_entry: a_remote_entry } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { .. } => LATER,
                    SyncAction::CreateDir { .. } => LATER,
                    SyncAction::AddFile { .. } => LATER,
                    SyncAction::UpdateFile { .. } => LATER,
                    SyncAction::DeleteFile { remote_entry: b_remote_entry } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_remote_entry.size.cmp(&a_remote_entry.size)
                    }
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
            SyncAction::PrereqDeleteDir { remote_path: a_remote_path } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { remote_path: b_remote_path } => {
                        // Deeper directories are deleted earlier, shallower directories later
                        // (Delete /foo/bar/baz -> Delete /foo/bar -> Delete /foo)
                        b_remote_path.components().count().cmp(&a_remote_path.components().count())
                    }
                    SyncAction::CreateDir { .. } => EARLIER,
                    SyncAction::AddFile { .. } => EARLIER,
                    SyncAction::UpdateFile { .. } => EARLIER,
                    SyncAction::DeleteFile { .. } => EARLIER,
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
            SyncAction::PrereqDeleteFile { remote_entry: a_remote_entry } => {
                match b {
                    SyncAction::PrereqDeleteFile { remote_entry: b_remote_entry } => {
                        // Bigger files are deleted earlier, smaller files later
                        // (Delete Gigabytes -> Delete Megabytes -> Delete Kilobytes)
                        b_remote_entry.size.cmp(&a_remote_entry.size)
                    }
                    SyncAction::PrereqDeleteDir { .. } => EARLIER,
                    SyncAction::CreateDir { .. } => EARLIER,
                    SyncAction::AddFile { .. } => EARLIER,
                    SyncAction::UpdateFile { .. } => EARLIER,
                    SyncAction::DeleteFile { .. } => EARLIER,
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
            SyncAction::UpdateFile {
                local_entry: a_local_entry,
                type_based_retardation_index: a_type_based_retardation_index,
                ..
            } => {
                match b {
                    SyncAction::PrereqDeleteFile { .. } => LATER,
                    SyncAction::PrereqDeleteDir { .. } => LATER,
                    SyncAction::CreateDir { .. } => LATER,
                    SyncAction::AddFile { .. } => LATER,
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
                    SyncAction::DeleteFile { .. } => EARLIER,
                    SyncAction::DeleteDir { .. } => EARLIER
                }
            }
        }
    })
}

// TODO: Implement
// Tight storage quota strategy (prioritize keeping used storage as low as possible at every point of the deployment)
// "Prioritize avoidance of storage quota overflows during deployment"
//
// 1. Create all missing directories (recursively - going from shallowest to deepest path)
// 2. Delete all files, from biggest to smallest
// 3. Delete all directories (recursively - going from deepest to shallowest path)
// 4. Replace updated files, going from biggest to smallest
// 5. Upload all completely NEW files, going from biggest to smallest
// fn order_sync_actions_survival_approach(actions: &mut Vec<SyncAction>) {
//     unimplemented!()
// }

pub fn perform_deploy(
    context_ref: &Arc<Context>,
    mut ftp_stream_wrapper: FtpStreamWrapper,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    {
        let mut deployment = context_ref.deploying.lock().unwrap();
        if deployment.active() {
            let message = translations.already_deploying_can_only_deploy_again_when_current_deployment_is_finished;
            return Err(message.to_string());
        } else {
            deployment.start();
        }
    }

    let context = Arc::clone(context_ref);
    let ftp_config_path = PathBuf::from(site.ftp_config.path_unchecked());
    let translations_moving = translations.clone();

    thread::spawn(move || {
        let mut report = |message: &str| {
            context.deploying.lock().unwrap().report(message);
        };

        if ftp_stream_wrapper.is_secure() {
            report(translations_moving.beginning_deployment_over_secure_ftp_tls_ssl);
        } else {
            report(translations_moving.beginning_deployment_over_insecure_ftp_unencrypted);
        }

        let mut actions = Vec::new();

        match collect_sync_actions(
            &mut actions,
            &mut ftp_stream_wrapper,
            &context.build_dir,
            &ftp_config_path,
            true
        ) {
            Ok(()) => {
                order_sync_actions_quality_approach(&mut actions);

                // TODO: Implement and make accessible through some option
                // order_sync_actions_survival_approach(&mut actions);

                report(translations_moving.deployment_steps_successfully_determined_file_synchronization_will_now_commence);

                match execute_sync_actions(
                    &actions,
                    &mut ftp_stream_wrapper,
                    &mut report,
                    &translations_moving
                ) {
                    Ok(()) => report(translations_moving.deployment_completed_successfully),
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

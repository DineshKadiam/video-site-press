// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};
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
use suppaftp::types::{Features, FileType};

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
    Add { remote_path: PathBuf },
    Pass,
    Replace { remote_entry: SyncEntry },
    Recurse { remote_entry: SyncEntry }
}

#[derive(Debug)]
struct SyncEntry {
    pub file_name: String,
    pub is_dir: bool,
    // yyyymmddHHMMSS (e.g. "20250509105623")
    pub modified: String,
    pub path: PathBuf
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

    pub fn feat(&mut self) -> FtpResult<Features> {
        trace!("feat");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.feat(),
            FtpStreamWrapper::Secure(stream) => stream.feat()
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

/// Create the directory and all recursive directories and files inside it
/// based on the passed local directory. Both `local_path` and `remote_path`
/// are absolute paths (in their respective context).
fn add_dir(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    local_path: &Path,
    remote_path: &Path
) -> Result<(), SyncError> {
    ftp_stream_wrapper.mkdir(&remote_path)?;

    for local_entry in get_local_entries(false, local_path)? {
        if local_entry.is_dir {
            add_dir(
                ftp_stream_wrapper,
                &local_entry.path,
                &remote_path.join(&local_entry.file_name)
            )?;
        } else {
            add_file(
                ftp_stream_wrapper,
                &local_entry,
                &remote_path.join(&local_entry.file_name)
            )?;
        }
    }

    Ok(())
}

/// Both `local_file` and `remote_file` are absolute paths (in their
/// respective context).
fn add_file(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    local_entry: &SyncEntry,
    remote_path: &Path
) -> Result<(), SyncError> {
    ftp_stream_wrapper.put_with_stream(&local_entry.path, remote_path)?;

    // https://en.wikipedia.org/wiki/File_Transfer_Protocol#Additional_commands
    // https://support.solarwinds.com/SuccessCenter/s/article/MFMT-FTP-command
    // https://forum.filezilla-project.org/viewtopic.php?t=15686
    let local_modified = &local_entry.modified;
    let remote_path = remote_path.display();
    let mfmt = format!("MFMT {local_modified} {remote_path}");

    ftp_stream_wrapper.custom_command(&mfmt)?;

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

            let sync_entry = SyncEntry {
                file_name,
                is_dir,
                modified,
                path
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

                let sync_entry = SyncEntry {
                    file_name,
                    is_dir,
                    modified,
                    path
                };

                Some(sync_entry)
            } else {
                None
            }
        })
        .collect::<Vec<SyncEntry>>();

    Ok(remote_entries)
}

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

        // TODO: Localize
        if ftp_stream_wrapper.is_secure() {
            report("Beginning deployment over secure FTP (TLS/SSL)");
        } else {
            report("Beginning deployment over insecure FTP (unencrypted)");
        }

        match sync_directory(
            &mut ftp_stream_wrapper,
            &context.build_dir,
            &ftp_config_path,
            true,
            &mut report,
            &translations_moving
        ) {
            // TODO: Translate
            Ok(()) => report("Deployment completed successfully"),
            Err(err) => report(&err.to_string())
        }

        let _ = ftp_stream_wrapper.quit();

        context.deploying.lock().unwrap().complete();
    });

    Ok(())
}

/// Compare local and remote directories and files within this directory and
/// perform the respective syncing actions (add, replace, recurse, remove,
/// pass) for each.
fn sync_directory(
    ftp_stream_wrapper: &mut FtpStreamWrapper,
    local_dir: &Path,
    remote_dir: &Path,
    // When we call this at the root/entry level of our sync operation we
    // ignore hidden files, as they might be owned by the server
    // infrastructure and should not be meddled with
    // (e.g. ".ftpquota" or ".htaccess" files)
    ignore_hidden: bool,
    report: &mut impl FnMut(&str),
    translations: &Translations
) -> Result<(), SyncError> {
    let local_entries = get_local_entries(ignore_hidden, local_dir)?;
    let mut remote_entries = get_remote_entries(ftp_stream_wrapper, ignore_hidden, remote_dir)?;

    for local_entry in local_entries {
        let remote_entry_with_same_name = remote_entries
            .iter()
            .position(|remote_entry| remote_entry.file_name == local_entry.file_name)
            .map(|index| remote_entries.swap_remove(index));

        // Determine sync action
        let sync_action = match remote_entry_with_same_name {
            Some(remote_entry) => {
                if local_entry.is_dir {
                    if remote_entry.is_dir {
                        SyncAction::Recurse { remote_entry }
                    } else {
                        SyncAction::Replace { remote_entry }
                    }
                } else if remote_entry.is_dir {
                    SyncAction::Replace { remote_entry }

                } else if remote_entry.modified != local_entry.modified {
                    SyncAction::Replace { remote_entry }
                } else {
                    SyncAction::Pass
                }
            }
            None => {
                let remote_path = remote_dir.join(&local_entry.file_name);
                SyncAction::Add { remote_path }
            }
        };

        // Perform sync action
        match sync_action {
            SyncAction::Add { remote_path } => {
                let message = translations.adding_xxx(&remote_path.to_string_lossy());
                report(&message);

                if local_entry.is_dir {
                    add_dir(
                        ftp_stream_wrapper,
                        &local_entry.path,
                        &remote_path
                    )?;
                } else {
                    add_file(
                        ftp_stream_wrapper,
                        &local_entry,
                        &remote_path
                    )?;
                }
            }
            SyncAction::Pass => (),
            SyncAction::Recurse { remote_entry } => {
                sync_directory(
                    ftp_stream_wrapper,
                    &local_entry.path,
                    &remote_entry.path,
                    false,
                    report,
                    translations
                )?;
            }
            SyncAction::Replace { remote_entry } => {
                let message = translations.replacing_xxx(&remote_entry.path.to_string_lossy());
                report(&message);

                if remote_entry.is_dir {
                    ftp_stream_wrapper.rmdir(&remote_entry.path)?;

                    add_file(
                        ftp_stream_wrapper,
                        &local_entry,
                        &remote_entry.path
                    )?;
                } else {
                    ftp_stream_wrapper.rm(&remote_entry.path)?;

                    if local_entry.is_dir {
                        add_dir(
                            ftp_stream_wrapper,
                            &local_entry.path,
                            &remote_entry.path
                        )?;
                    } else {
                        add_file(
                            ftp_stream_wrapper,
                            &local_entry,
                            &remote_entry.path
                        )?;
                    }
                }
            }
        }
    }

    // Remove leftover remote entries that don't correspond to anything we
    // have locally.
    for remote_entry in remote_entries {
        let message = translations.deleting_xxx(&remote_entry.path.to_string_lossy());
        report(&message);

        if remote_entry.is_dir {
            ftp_stream_wrapper.rmdir(&remote_entry.path)?;
        } else {
            ftp_stream_wrapper.rm(&remote_entry.path)?;
        }
    }

    Ok(())
}

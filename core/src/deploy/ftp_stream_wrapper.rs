// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use log::trace;
use suppaftp::{
    FtpResult,
    FtpStream,
    NativeTlsConnector,
    NativeTlsFtpStream,
    Status
};
use suppaftp::native_tls::TlsConnector;
use suppaftp::types::FileType;

use crate::Site;

use super::SyncError;

const DEFAULT_FTP_PORT: usize = 21;

// TODO: Not the case anymore, we could now access the needed traits
// (requires reimplementation of a few things here of course)
/// The crate we use for FTP interaction (suppaftp) does not publicly expose
/// the traits we would need to generically implement insecure/secure FTP
/// stream interaction, therefore we put both stream types into our own
/// wrapper and reimplement a generic function call for each FTP command we
/// use from the crate.
pub enum FtpStreamWrapper {
    Insecure(FtpStream),
    Secure(NativeTlsFtpStream)
}

impl FtpStreamWrapper {
    pub fn connect(site: &Site) -> Result<FtpStreamWrapper, SyncError> {
        let password = site.ftp_config.password_unchecked();
        let port = site.ftp_config.port.unwrap_or(DEFAULT_FTP_PORT);
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

    pub fn cwd(&mut self, path: impl AsRef<Path>) -> FtpResult<()> {
        let path_str = path.as_ref().to_string_lossy();

        trace!("cwd {path_str}");

        match self {
            FtpStreamWrapper::Insecure(stream) => stream.cwd(path_str),
            FtpStreamWrapper::Secure(stream) => stream.cwd(path_str)
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

        // TODO: In the SSH sync implementation, BufReader based file transfer
        // has proven to be massively slower (than just doing file.write_all(...))
        // even for rather small files (e.g. 52MB). We should re-evaluate if we
        // can make these gains here too (but there might have been a reason to go
        // for the BufReader approach here, so check/test carefully).

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

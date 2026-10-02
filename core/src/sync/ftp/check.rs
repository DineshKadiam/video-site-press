// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Cursor;
use std::path::{Path};

use indoc::indoc;
use suppaftp::{ImplFtpStream, TlsStream};

use hyper8_translations::Translations;
use hyper8_util::nanoid;

use crate::{
    Site,
    SitePath,
    SyncConfigCheckStep,
    SyncError
};

use super::DEFAULT_FTP_PORT;

const TEST_FILE_NAME: &str = ".hyper8_ftp_sync_config_check.txt";
const TEST_FILE_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://hyper8.org) to check if the
    configured server and path for FTP synchronization matches up with the
    configured public website URL. It serves no further purpose anymore and
    may safely be deleted.
");

/// First tries to connect to the remote, thereby verifying the credentials.
/// Then uploads a test file with a random token to the configured remote
/// directory, verifying it is present and writeable. Then tries to fetch
/// that same file through the public url of the website (if configured),
/// thereby confirming that the website is set up correctly and no other
/// website (or other data) will be wiped due to a wrongly set remote path.
/// Regardless of failure or success of the check, the test file is removed
/// from the remote again, leaving a clean state.
pub fn check_sync_config<S: TlsStream>(
    connect: impl Fn(&str, &Site) -> Result<ImplFtpStream<S>, SyncError>,
    prompted_password: Option<&str>,
    site: &Site,
    translations: &Translations
) -> Vec<SyncConfigCheckStep> {
    let mut steps = Vec::new();

    // Login

    let port = site.ftp_config.port.unwrap_or(DEFAULT_FTP_PORT);
    let server = site.ftp_config.server_unchecked();
    let user = site.ftp_config.user_unchecked();

    let password = prompted_password
        .or(site.ftp_config.password.as_deref())
        .unwrap();

    let mut login = SyncConfigCheckStep::new(translations.login);

    login.context(format!("{user}@{server}:{port}"));

    let mut impl_ftp_stream = match connect(password, site) {
        Ok(impl_ftp_stream) => impl_ftp_stream,
        Err(err) => {
            login.message(err);

            steps.push(login.fail());
            return steps;
        }
    };

    steps.push(login.pass());

    // Open remote directory

    let remote_dir = site.ftp_config.path_unchecked();

    let mut open_remote_directory = SyncConfigCheckStep::new(translations.open_remote_directory);

    open_remote_directory.context(remote_dir);

    // Check if the configured remote directory exists (by changing into it)
    if let Err(remote_dir_err) = impl_ftp_stream.cwd(&remote_dir) {
        // If the configured remote directory does not exist, try changing
        // into each of its ancestor directories (going from deepest to
        // shallowest) in order to determine and report the first existing
        // directory that is part of the user's configured remote directory
        // path.
        while let Some(ancestor_dir) = Path::new(remote_dir).parent() {
            if impl_ftp_stream.cwd(ancestor_dir.to_string_lossy()).is_ok() {
                let message = translations.remote_dir_does_not_exist_ancestor_dir_found(
                    &ancestor_dir.to_string_lossy(),
                    remote_dir,
                    &remote_dir_err.to_string()
                );

                open_remote_directory.message(message);

                steps.push(open_remote_directory.fail());
                return steps;
            }
        }

        let message = translations.neither_remote_dir_xxx_nor_any_ancestor_dir_found(
            remote_dir,
            &remote_dir_err.to_string()
        );

        open_remote_directory.message(message);

        steps.push(open_remote_directory.fail());
        return steps;
    }

    steps.push(open_remote_directory.pass());

    // Create test file

    let probe_signature = nanoid();
    let probe_content = format!("{TEST_FILE_TEXT}\n\n{probe_signature}");
    let remote_probe_path = Path::new(remote_dir).join(TEST_FILE_NAME);

    let mut create_test_file = SyncConfigCheckStep::new(translations.create_test_file);

    create_test_file.context(remote_probe_path.display().to_string());

    let mut reader = Cursor::new(probe_content.as_bytes());

    // Perform write
    if let Err(err) = impl_ftp_stream.put_file(remote_probe_path.to_string_lossy(), &mut reader) {
        create_test_file.message(err.to_string());
        steps.push(create_test_file.fail());
        return steps;
    }

    steps.push(create_test_file.pass());

    // Download and compare test file

    let mut download_and_compare_test_file = SyncConfigCheckStep::new(translations.download_and_compare_test_file);

    if let Some(base_url) = &site.base_url {
        let public_probe_url = site.absolute_url_file_unchecked(
            TEST_FILE_NAME,
            &SitePath::root_path()
        );

        download_and_compare_test_file.context(&public_probe_url);

        // Prepare GET request for the test file through its public address,
        // passing http headers to explicitly request a non-cached response.
        let public_probe_request = minreq::get(public_probe_url.clone())
            .with_header("Cache-Control", "no-cache, no-store, must-revalidate")
            .with_header("Pragma", "no-cache")
            .with_header("Expires", "0");

        // Fetch probe file through public url and return result
        match public_probe_request.send() {
            Ok(response) => {
                match response.as_str() {
                    Ok(body) => {
                        if body == probe_content {
                            let message = translations.the_public_site_address_xxx_correctly_serves_content_in_remote_dir_xxx_text(
                                remote_dir,
                                &base_url.representation
                            );

                            download_and_compare_test_file.message(message);

                            steps.push(download_and_compare_test_file.pass());
                        } else {
                            // When printing the mismatched probe content next
                            // to each other for the user to troubleshoot we
                            // truncate each to the length of the probe
                            // content plus some overhead, because this is
                            // usually the range in which the essential
                            // insights for troubleshooting lie.
                            let truncate_len = probe_content.chars().count() + 64;

                            let truncate = |text: &str| -> String {
                                match text.char_indices().nth(truncate_len) {
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

                            download_and_compare_test_file.message(message);

                            steps.push(download_and_compare_test_file.fail());
                        }
                    }
                    Err(err) => {
                        let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                            &err.to_string(),
                            &remote_probe_path.to_string_lossy(),
                            &public_probe_url
                        );

                        download_and_compare_test_file.message(message);

                        steps.push(download_and_compare_test_file.fail());
                    }
                }
            }
            Err(err) => {
                let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                    &err.to_string(),
                    &remote_probe_path.to_string_lossy(),
                    &public_probe_url
                );

                download_and_compare_test_file.message(message);

                steps.push(download_and_compare_test_file.fail());
            }
        }
    } else {
        let message = translations.the_public_site_address_is_not_configured_this_prevents_checking_text;
        download_and_compare_test_file.message(message);
        steps.push(download_and_compare_test_file.skip());
    }

    // Remove test file

    let mut remove_test_file = SyncConfigCheckStep::new(translations.remove_test_file);

    remove_test_file.context(remote_probe_path.display().to_string());

    match impl_ftp_stream.rm(remote_probe_path.to_string_lossy()) {
        Ok(()) => {
            let _ = impl_ftp_stream.quit();
            steps.push(remove_test_file.pass());
        }
        Err(err) => {
            remove_test_file.message(err.to_string());
            steps.push(remove_test_file.fail());
        }
    }

    steps
}

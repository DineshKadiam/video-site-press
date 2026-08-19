// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;
use std::path::Path;

use indoc::indoc;
use ssh2::DisconnectCode;

use hyper8_translations::Translations;
use hyper8_util::nanoid;

use crate::{
    Site,
    SitePath,
    SyncConfigCheckStep
};

use super::DEFAULT_SSH_PORT;
use super::connect;

const TEST_FILE_NAME: &str = ".hyper8_ssh_sync_config_check.txt";
const TEST_FILE_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://hyper8.org) to check if the
    configured server and path for SSH synchronization matches up with the
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
pub fn check_sync_config(
    site: &Site,
    translations: &Translations
) -> Vec<SyncConfigCheckStep> {
    let mut steps = Vec::new();

    // Login

    let port = site.ssh_config.port.unwrap_or(DEFAULT_SSH_PORT);
    let server = site.ssh_config.server_unchecked();
    let user = site.ssh_config.user_unchecked();

    let mut login = SyncConfigCheckStep::new(translations.login);

    login.context(format!("{user}@{server}:{port}"));

    let (session, sftp) = match connect(site, translations) {
        Ok(session_sftp) => session_sftp,
        Err(err) => {
            login.message(err);

            steps.push(login.fail());
            return steps;
        }
    };

    steps.push(login.pass());

    // Open remote directory

    let remote_dir = site.ssh_config.path_unchecked();

    let mut open_remote_directory = SyncConfigCheckStep::new(translations.open_remote_directory);

    open_remote_directory.context(remote_dir);

    // Check if the configured remote directory exists and is a file
    match sftp.stat(Path::new(remote_dir)) {
        Ok(file_stat) => {
            if !file_stat.is_dir() {
                let message = translations.remote_dir_xxx_is_not_a_directory(remote_dir);

                open_remote_directory.message(message);

                steps.push(open_remote_directory.fail());
                return steps;
            }
        }
        Err(remote_dir_err) => {
            // If the configured remote directory does not exist, try changing
            // into each of its ancestor directories (going from deepest to
            // shallowest) in order to determine and report the first existing
            // directory that is part of the user's configured remote directory
            // path.
            while let Some(ancestor_dir) = Path::new(remote_dir).parent() {
                if sftp.stat(&ancestor_dir).is_ok() {
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
    }

    steps.push(open_remote_directory.pass());

    // Create test file

    let probe_signature = nanoid();
    let probe_content = format!("{TEST_FILE_TEXT}\n\n{probe_signature}");
    let remote_probe_path = Path::new(remote_dir).join(TEST_FILE_NAME);

    let mut create_test_file = SyncConfigCheckStep::new(translations.create_test_file);

    create_test_file.context(remote_probe_path.display().to_string());

    // Perform write
    match sftp.create(&remote_probe_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(probe_content.as_bytes()) {
                create_test_file.message(err.to_string());
                steps.push(create_test_file.fail());
                return steps;
            }
        }
        Err(err) => {
            create_test_file.message(err.to_string());
            steps.push(create_test_file.fail());
            return steps;
        }
    }

    steps.push(create_test_file.pass());

    // Download and compare test file

    let mut download_and_compare_test_file = SyncConfigCheckStep::new(translations.download_and_compare_test_file);

    // TODO: Asides the parametrized test file name & content this part of the
    // check could be 100% reused between FTP and SSH now
    if let Some(base_url) = &site.base_url {
        let public_probe_url = site.absolute_url_file_unchecked(
            TEST_FILE_NAME,
            &SitePath::root_path()
        );

        download_and_compare_test_file.context(&public_probe_url);

        // Fetch probe file through public url and return result
        match minreq::get(public_probe_url.clone()).send() {
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

                            download_and_compare_test_file.message(message);

                            steps.push(download_and_compare_test_file.fail());
                        }
                    }
                    Err(err) => {
                        let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                            &err.to_string(),
                            &public_probe_url,
                            &remote_probe_path.to_string_lossy()
                        );

                        download_and_compare_test_file.message(message);

                        steps.push(download_and_compare_test_file.fail());
                    }
                }
            }
            Err(err) => {
                let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                    &err.to_string(),
                    &public_probe_url,
                    &remote_probe_path.to_string_lossy()
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

    match sftp.unlink(&remote_probe_path) {
        Ok(()) => {
            let _ = session.disconnect(
                Some(DisconnectCode::ByApplication),
                "",
                None
            );

            steps.push(remove_test_file.pass());
        }
        Err(err) => {
            remove_test_file.message(err.to_string());
            steps.push(remove_test_file.fail());
        }
    }

    steps
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

use indoc::indoc;

use hyper8_translations::Translations;

use crate::{Context, Site, SitePath};
use crate::util::nanoid;

const RSYNC_CONFIG_PROBE_FILE_NAME: &str = ".hyper8_rsync_config_probe.txt";
const RSYNC_CONFIG_PROBE_TEXT: &str = indoc!("
    This file was created by Hyper 8 (https://simonrepp.com/hyper8) to verify
    the configured server and path for rsync deployment matches up with the
    configured website url. It serves no further purpose anymore and may
    safely be deleted.
");

/// Windows = Dummy implementation - we need a packaged rsync for this, which is not so trivial
pub const RSYNC: &str = if cfg!(windows) { "rsync.exe" } else { "rsync" };

/// Uploads a file named ".hyper8_rsync_config_probe.txt", which also contains a random token,
/// to the remote, thereby first verifying that the deployment configuration is actually
/// correct and working. Then, right next, tries to fetch that same file through the public
/// url of the website (e.g. https://example.com/.hyper8_rsync_config_probe.txt), thereby confirming
/// that the website is set up correctly (although that is a side effect), but more importantly
/// ensuring that no other website (or remote content of any kind) is accidentally wiped if the
/// remote path has been accidentally set incorrectly. Regardless of failure or success of the
/// check, the probe file is removed from the remote again, leaving a clean state.
pub fn config_probe(
    context: &Context,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    let probe_signature = nanoid();
    let probe_content = format!("{RSYNC_CONFIG_PROBE_TEXT}\n\n{probe_signature}");

    let local_probe_path = context.build_dir.join(RSYNC_CONFIG_PROBE_FILE_NAME);
    let remote_probe_destination = format!(
        "{user}@{server}:{path}",
        path = Path::new(site.ssh_config.path_unchecked()).join(RSYNC_CONFIG_PROBE_FILE_NAME).to_string_lossy(),
        server = site.ssh_config.server_unchecked(),
        user = site.ssh_config.user_unchecked()
    );

    fs::write(&local_probe_path, &probe_content).unwrap();

    let mut upload_probe_command = Command::new(RSYNC);

    upload_probe_command
        .arg(&local_probe_path)
        .arg(&remote_probe_destination);

    let upload_probe_result = upload_probe_command.output();

    let _ = fs::remove_file(&local_probe_path);

    match upload_probe_result {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!("The rsync child process for syncing the config probe to the remote errored. ({output:?})"))
            }
        }
        Err(err) => return Err(format!("The rsync child process for syncing the config probe to the remote could not be executed. ({err})"))
    }

    let public_probe_url = site.absolute_url_file_unchecked(RSYNC_CONFIG_PROBE_FILE_NAME, &SitePath::root_path());

    let remove_probe_and_return = |previous_result: Result<(), String>| -> Result<(), String> {
        let remote_deploy_path = format!(
            "{user}@{server}:{path}",
            path = Path::new(site.ssh_config.path_unchecked()).join("./").to_string_lossy(),
            server = site.ssh_config.server_unchecked(),
            user = site.ssh_config.user_unchecked()
        );

        let mut remove_probe_command = Command::new(RSYNC);

        // We're creatively combining rsync's options here to sync a single file to
        // the remote which does not exist locally anymore, it therefore is removed.
        remove_probe_command
            .arg("--delete")
            .arg("--dirs")
            .arg(format!("--include={RSYNC_CONFIG_PROBE_FILE_NAME}")) // must appear before --exclude (!) otherwise removal does not happen
            .arg("--exclude=*")
            .arg(context.build_dir.join("./"))
            .arg(&remote_deploy_path);

        // If removing the probe fails when we already had a previous
        // error, we prioritize returning the earlier error, as it
        // likely provides a more meaningful context.
        match remove_probe_command.output() {
            Ok(output) => {
                if output.status.success() || previous_result.is_err() {
                    previous_result
                } else {
                    Err(format!("The rsync child process for syncing the config probe to the remote errored. ({output:?})"))
                }
            }
            Err(file_removal_err) => {
                if previous_result.is_err() {
                    previous_result
                } else {
                    Err(file_removal_err.to_string())
                }
            }
        }
    };

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

                        let remote_probe_path = format!("{}/{RSYNC_CONFIG_PROBE_FILE_NAME}", site.ssh_config.path_unchecked());

                        let err = translations.the_test_file_contains_xxx_but_the_response_contained_xxx(
                            &truncate(&probe_content),
                            &truncate(body)
                        );

                        let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                            &remote_probe_path,
                            &public_probe_url,
                            &err
                        );
                        remove_probe_and_return(Err(message))
                    }
                }
                Err(err) => {
                    let remote_probe_path = format!("{}/{RSYNC_CONFIG_PROBE_FILE_NAME}", site.ssh_config.path_unchecked());
                    let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                        &remote_probe_path,
                        &public_probe_url,
                        &err.to_string()
                    );
                    remove_probe_and_return(Err(message))
                }
            }
        }
        Err(err) => {
            let remote_probe_path = format!("{}/{RSYNC_CONFIG_PROBE_FILE_NAME}", site.ssh_config.path_unchecked());
            let message = translations.test_file_successfully_uploaded_but_remote_verification_failed_message(
                &remote_probe_path,
                &public_probe_url,
                &err.to_string()
            );
            remove_probe_and_return(Err(message))
        }
    }
}

pub fn start_deploy(
    context_ref: &Arc<Context>,
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

    let remote_deploy_path = format!(
        "{user}@{server}:{path}",
        path = Path::new(site.ssh_config.path_unchecked()).join("./").to_string_lossy(),
        server = site.ssh_config.server_unchecked(),
        user = site.ssh_config.user_unchecked()
    );

    let mut command = Command::new(RSYNC);

    command.arg("-avP");
    command.arg("--delete");
    command.arg(context_ref.build_dir.join("./"));
    command.arg(remote_deploy_path);

    command.stdout(Stdio::piped());

    let context = Arc::clone(context_ref);

    thread::spawn(move || {
        match command.spawn() {
            Ok(mut child) => {
                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        let message = "Could not capture stdout for rsync child process.";
                        context.deploying.lock().unwrap().report(message);
                        return;
                    }
                };

                let stdout_reader = BufReader::new(stdout);

                for result in stdout_reader.lines() {
                    if let Ok(line) = result {
                        context.deploying.lock().unwrap().report(&line);
                    }
                }

                let exit_status = match child.wait() {
                    Ok(exit_status) => exit_status,
                    Err(_) => {
                        let message = "The rsync child process could not be waited for.";
                        context.deploying.lock().unwrap().report(message);
                        return;
                    }
                };

                if !exit_status.success() {
                    let error_code = match exit_status.code() {
                        Some(code) => format!("error code {code}"),
                        None => String::from("probably signal-terminated")
                    };
                    let message = format!("The rsync child process terminated unsuccessfully ({error_code})");
                    context.deploying.lock().unwrap().report(&message);
                }
            }
            Err(err) => {
                let message = format!("The rsync child process could not be executed ({err})");
                context.deploying.lock().unwrap().report(&message);
            }
        }

        context.deploying.lock().unwrap().complete();
    });

    Ok(())
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

use crate::{Context, Site};
use crate::util::nanoid;

const PROBE_FILE_NAME: &str = ".hyper8_config_probe.txt";
const PROBE_TEXT: &str = "\
This file was created by Hyper 8 to verify the configured server and path\
for deployment matches up with the configured website url. It serves no\
further purpose anymore and may safely be deleted.\
";

/// Windows = Dummy implementation - we need a packaged rsync for this, which is not so trivial
pub const RSYNC: &str = if cfg!(windows) { "rsync.exe" } else { "rsync" };

pub struct DeploymentState {
    pub completed_at: Option<SystemTime>,
    pub feedback: String,
    pub started_at: Option<SystemTime>
}

impl DeploymentState {
    pub fn active(&self) -> bool {
        self.completed_at.is_none() && self.started_at.is_some()
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(SystemTime::now());
    }

    pub fn new() -> DeploymentState {
        DeploymentState {
            completed_at: None,
            feedback: String::new(),
            started_at: None
        }
    }

    pub fn report(&mut self, line: &str) {
        if !self.feedback.is_empty() {
            self.feedback.push('\n');
        }

        self.feedback.push_str(line);
    }

    pub fn start(&mut self) {
        self.completed_at = None;
        self.feedback.clear();
        self.started_at = Some(SystemTime::now());
    }
}

/// Uploads a file named ".hyper8_config_probe.txt", which also contains a random token,
/// to the remote, thereby first verifying that the deployment configuration is actually
/// correct and working. Then, right next, tries to fetch that same file through the public
/// url of the website (e.g. https://example.com/.hyper8_config_probe.txt), thereby confirming
/// that the website is set up correctly (although that is a side effect), but more importantly
/// ensuring that no other website (or remote content of any kind) is accidentally wiped if the
/// remote path has been accidentally set incorrectly. Regardless of failure or success of the
/// check, the probe file is removed from the remote again, leaving a clean state.
pub fn config_probe(context: &Context, site: &Site) -> Result<(), String> {
    let probe_signature = nanoid();
    let probe_content = format!("{PROBE_TEXT}\n\n{probe_signature}");

    let local_probe_path = context.build_dir.join(PROBE_FILE_NAME);
    let remote_probe_path = format!(
        "{user}@{server}:{path}",
        path = Path::new(site.deploy_config.path.as_ref().unwrap()).join(PROBE_FILE_NAME).to_string_lossy(),
        server = site.deploy_config.server.as_ref().unwrap(),
        user = site.deploy_config.user.as_ref().unwrap()
    );

    fs::write(&local_probe_path, &probe_content).unwrap();

    let mut upload_probe_command = Command::new(RSYNC);

    upload_probe_command
        .arg(&local_probe_path)
        .arg(&remote_probe_path);

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

    let public_probe_url = site.base_url.as_ref().unwrap().value.join(PROBE_FILE_NAME).unwrap();

    match minreq::get(public_probe_url.clone()).send() {
        Ok(response) => {
            let remote_deploy_path = format!(
                "{user}@{server}:{path}",
                path = Path::new(site.deploy_config.path.as_ref().unwrap()).join("./").to_string_lossy(),
                server = site.deploy_config.server.as_ref().unwrap(),
                user = site.deploy_config.user.as_ref().unwrap()
            );

            let mut remove_probe_command = Command::new(RSYNC);

            // We're creatively combining rsync's options here to sync a single file to
            // the remote which does not exist locally anymore, it therefore is removed.
            remove_probe_command
                .arg("--delete")
                .arg("--dirs")
                .arg(format!("--include={PROBE_FILE_NAME}")) // must appear before --exclude (!) otherwise removal does not happen
                .arg("--exclude=*")
                .arg(context.build_dir.join("./"))
                .arg(&remote_deploy_path);

            let _ = remove_probe_command.output();

            if response.as_str().is_ok_and(|body| body == probe_content) {
                Ok(())
            } else {
                Err(format!("The config probe was successfully uploaded to {remote_probe_path} but its presence could not be verified through the public interface {public_probe_url}. (Double-check that both the base url and the deployment configuration are correct)"))
            }
        }
        Err(err) => Err(format!("The config probe was successfully uploaded to {remote_probe_path} but not obtainable through the public interface {public_probe_url}. ({err})"))
    }
}

pub fn perform_deploy(context_ref: &Arc<Context>, site: &Site) -> Result<(), String> {
    {
        let mut deployment = context_ref.deploying.lock().unwrap();
        if deployment.active() {
            let language = context_ref.get_editor_language();
            let message = language.translations.already_deploying_can_only_deploy_again_when_current_deployment_is_finished;
            return Err(message.to_string());
        } else {
            deployment.start();
        }
    }

    let context = Arc::clone(context_ref);

    let remote_deploy_path = format!(
        "{user}@{server}:{path}",
        path = Path::new(site.deploy_config.path.as_ref().unwrap()).join("./").to_string_lossy(),
        server = site.deploy_config.server.as_ref().unwrap(),
        user = site.deploy_config.user.as_ref().unwrap()
    );

    let mut command = Command::new(RSYNC);

    command.arg("-avP");
    command.arg("--delete");
    command.arg(context.build_dir.join("./"));
    command.arg(remote_deploy_path);

    command.stdout(Stdio::piped());

    thread::spawn(move || {
        // TODO: Pass errors back to interface and user
        match command.spawn() {
            Ok(mut child) => {
                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        eprint!("Could not capture stdout for rsync child process.");
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
                        eprint!("The rsync child process could not be waited for.");
                        return;
                    }
                };

                if !exit_status.success() {
                    let error_code = match exit_status.code() {
                        Some(code) => format!("error code {code}"),
                        None => String::from("probably signal-terminated")
                    };
                    eprint!("The rsync child process terminated unsuccessfully ({error_code})");
                }
            }
            Err(err) => {
                eprint!("The rsync child process could not be executed.");
                eprint!("{err}");
            }
        }

        context.deploying.lock().unwrap().complete();
    });

    Ok(())
}
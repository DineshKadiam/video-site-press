use std::fs;
use std::path::Path;
use std::process::Command;

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

/// Uploads a file named ".hyper8_config_probe.txt", which also contains a random token,
/// to the remote, thereby first verifying that the deployment configuration is actually
/// correct and working. Then, right next, tries to fetch that same file through the public
/// url of the website (e.g. https://example.com/.hyper8_config_probe.txt), thereby confirming
/// that the website is set up correctly (although that is a side effect), but more importantly
/// ensuring that no other website (or remote content of any kind) is accidentally wiped if the
/// remote path has been accidentally set incorrectly. If the probe fails in any way, the probe
/// file is removed from the remote again, leaving a clean state. If the probe succeeds the probe
/// file is left, as it is anyway removed through the consecutive deployment operation.
fn config_probe(context: &Context, site: &Site) -> Result<(), String> {
    let probe_signature = nanoid();
    let probe_content = format!("{PROBE_TEXT}\n\n{probe_signature}");

    let local_probe_path = context.build_dir.join(&PROBE_FILE_NAME);
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
            if response.as_str().is_ok_and(|body| body == probe_content) {
                // Note that we are not removing the probe on the remote here - we already removed it locally
                // in the build dir and the following deploy operation will take care of the removal on the remote.
                Ok(())
            } else {
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
                    .arg(&format!("--include={PROBE_FILE_NAME}")) // must appear before --exclude (!) otherwise removal does not happen
                    .arg("--exclude=*")
                    .arg(&context.build_dir.join("./"))
                    .arg(&remote_deploy_path);

                let _ = remove_probe_command.output();

                Err(format!("The config probe was successfully uploaded to {remote_probe_path} but its presence could not be verified through the public interface {public_probe_url}. (Double-check that both the base url and the deployment configuration are correct)"))
            }
        }
        Err(err) => return Err(format!("The config probe was successfully uploaded to {remote_probe_path} but not obtainable through the public interface {public_probe_url}. ({err})"))
    }
}

pub fn perform_deploy(context: &Context) {    
    let site = context.get_site();

    // TODO: All of these errors (currently: panics) should be gracefully communicated through the browser interface

    if !site.base_url.is_some() {
        panic!("Base url is not set, therefore the deploy config cannot be probed, therefore deploying is not permitted for safety reasons.");
    }

    if !site.deploy_config.complete() {
        panic!("Deploy config is not complete, can not deploy.");
    }

    if let Err(err) = config_probe(context, &site) {
        panic!("The deployment configuration could not be verified: {err}");
    }

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

    // TODO: Pass errors back to interface and user
    match command.output() {
        Ok(output) => {
            if !output.status.success() {
                eprint!("{output:?}");
            }
        }
        Err(err) => {
            eprint!("The rsync child process could not be executed.");
            eprint!("{err}");
        }
    }
}
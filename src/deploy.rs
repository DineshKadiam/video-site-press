use std::process::Command;

use crate::Context;

/// Windows = Dummy implementation - we need a packaged rsync for this, which is not so trivial
pub const RSYNC: &str = if cfg!(windows) { "rsync.exe" } else { "rsync" };

pub fn perform_deploy(context: &Context) {    
    let mut command = Command::new(RSYNC);

    command.arg("-avP");
    command.arg("--delete");
    command.arg(context.build_dir.join("./"));
    command.arg("deploy/"); // TODO: Local dummy target for now

    // TODO: Disarmed for now
    // match command.output() {
    //     Ok(output) => {
    //         if !output.status.success() {
    //             eprint!("{output:?}");
    //         }
    //     }
    //     Err(err) => {
    //         eprint!("The rsync child process could not be executed.");
    //         eprint!("{err}");
    //     }
    // }
}
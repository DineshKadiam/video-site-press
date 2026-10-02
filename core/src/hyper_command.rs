// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::OsStr;
use std::process::Command;

// TODO: Note that we have observed that (maybe due to the way that tauri
// initializes the execution/window context) we don't actually have to set
// the CREATE_NO_WINDOW flag (windows don't seem to open either way),
// but for the sake of robustness and future developments we're keeping
// this abstraction around.

/// This is a thin wrapper for the intialization of rust's native Command
/// type. On Windows it creates the command and then sets the
/// CREATE_NO_WINDOW creation flag in order to prevent the process from
/// opening a window. On all other platforms it delegates 1:1 to rust's
/// Command::new itself, not adding any additional behavior.
pub struct HyperCommand;

impl HyperCommand {
    #[cfg(not(target_os = "windows"))]
    pub fn new(program: impl AsRef<OsStr>) -> Command {
        Command::new(program)
    }

    #[cfg(target_os = "windows")]
    pub fn new(program: impl AsRef<OsStr>) -> Command {
        use std::os::windows::process::CommandExt;
        use winapi::um::winbase::CREATE_NO_WINDOW;

        let mut command = Command::new(program);

        // Prevent a Windows command prompt window from opening for the external
        // command using the CREATE_NO_WINDOW process creation flag.
        // (see https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags#flags)
        command.creation_flags(CREATE_NO_WINDOW);

        command
    }
}

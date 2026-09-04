// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anstream::{eprint, eprintln};
use chrono::DateTime;
use chrono::offset::Local;

use hyper8_core::{
    Context,
    Site,
    SyncEventActionKind,
    SyncEventKind,
    SyncProtocol
};
use hyper8_core::start_sync;
use hyper8_translations::Translations;
use hyper8_util::human_size_delta;

use crate::style::{
    BLACK_ON_BRIGHT_YELLOW,
    BRIGHT_BLACK,
    BRIGHT_YELLOW,
    CYAN,
    MAGENTA,
    RED
};
use crate::sync_config;

pub fn sync(
    context: &Arc<Context>,
    non_interactive: bool,
    protocol: SyncProtocol,
    simulate: bool,
    translations: &Translations
) -> ExitCode {
    let site = context.get_site();

    if !simulate && !non_interactive {
        let t_really_sync = translations.really_sync;
        let base_url = &site.base_url.as_ref().unwrap().representation;
        eprintln!("{t_really_sync} {BRIGHT_BLACK}{base_url}{BRIGHT_BLACK:#}");
        eprint!("{BRIGHT_YELLOW}{}{BRIGHT_YELLOW:#} ", translations.type_y_and_press_enter_to_confirm_anything_else_will_abort);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() != "y" {
                    return ExitCode::SUCCESS;
                }
            }
            Err(err) => {
                eprintln!("{RED}{err}{RED:#}");
                return ExitCode::FAILURE;
            }
        }
    }

    let prompted_password = match sync_config::ensure_config(
        non_interactive,
        protocol,
        &site,
        translations
    ) {
        Ok(prompted_password) => prompted_password,
        Err(()) => return ExitCode::FAILURE
    };

    match execute_sync_blocking_cli(
        context,
        prompted_password.as_deref(),
        protocol,
        simulate,
        &site,
        &translations
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Performs live or simulated sync in a synchronous fashion, reporting
/// back all messages and errors to the terminal. Asks for explicit
/// (opt-in) confirmation before executing a live sync.
fn execute_sync_blocking_cli(
    context: &Arc<Context>,
    prompted_password: Option<&str>,
    protocol: SyncProtocol,
    simulate: bool,
    site: &Site,
    translations: &Translations
) -> Result<(), String> {
    start_sync(
        context,
        prompted_password,
        protocol,
        simulate,
        site,
        translations
    )?;

    let mut log_lines_seen = 0;

    loop {
        if let Some(sync_state) = &*context.sync_state.lock().unwrap() {
            let log = &sync_state.log;

            let simulated = if sync_state.simulation {
                let t_simulation = translations.simulation;
                format!("{BLACK_ON_BRIGHT_YELLOW} {t_simulation} {BLACK_ON_BRIGHT_YELLOW:#} ")
            } else {
                String::new()
            };

            if log.len() > log_lines_seen {
                for event in &log[log_lines_seen..] {
                    let message = match &event.kind {
                        SyncEventKind::Action { kind, path } => {
                            match kind {
                                SyncEventActionKind::AddFile { size_delta } => {
                                    let size_formatted = human_size_delta(*size_delta);
                                    let t_add_file = translations.add_file;
                                    format!(r#"{CYAN}[{t_add_file}]{CYAN:#} {BRIGHT_BLACK}{size_formatted}{BRIGHT_BLACK:#} {path}"#)
                                }
                                SyncEventActionKind::CreateDir => {
                                    let t_create_directory = translations.create_directory;
                                    format!(r#"{CYAN}[{t_create_directory}]{CYAN:#} {path}"#)
                                }
                                SyncEventActionKind::DeleteDir => {
                                    let t_delete_directory = translations.delete_directory;
                                    format!(r#"{MAGENTA}[{t_delete_directory}]{MAGENTA:#} {path}"#)
                                }
                                SyncEventActionKind::DeleteFile { size_delta } => {
                                    let size_formatted = human_size_delta(*size_delta);
                                    let t_delete_file = translations.delete_file;
                                    format!(r#"{MAGENTA}[{t_delete_file}]{MAGENTA:#} {BRIGHT_BLACK}{size_formatted}{BRIGHT_BLACK:#} {path}"#)
                                }
                                SyncEventActionKind::UpdateFile { size_delta } => {
                                    let size_formatted = human_size_delta(*size_delta);
                                    let t_update_file = translations.update_file;
                                    format!(r#"{BRIGHT_YELLOW}[{t_update_file}]{BRIGHT_YELLOW:#} {BRIGHT_BLACK}{size_formatted}{BRIGHT_BLACK:#} {path}"#)
                                }
                            }
                        }
                        SyncEventKind::Error(message) => {
                            format!("{RED}{message}{RED:#}")
                        }
                        SyncEventKind::Generic(message) => {
                            message.to_string()
                        }
                    };

                    let progress = if let Some(step) = event.step {
                        let steps = sync_state.steps.unwrap();
                        format!("({step}/{steps}) ")
                    } else {
                        String::new()
                    };

                    let started_at_datetime: DateTime<Local> = event.started_at.into();
                    let started_at_formatted = started_at_datetime.format("%T");
                    let started_at = format!("{BRIGHT_BLACK}[{started_at_formatted}]{BRIGHT_BLACK:#}");

                    eprintln!("{simulated}{started_at} {progress}{message}");
                }

                log_lines_seen = log.len();
            }

            if sync_state.finished() {
                break;
            }
        }

        // TODO: Maybe we can switch to a generics based system that utilizes
        // a different reporting implementation between desktop and cli, i.e.
        // CLI could potentially just print to terminal rather than passing
        // around messages with any sorts of locks involved? (see faircamp
        // where we have such an implementation)
        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}

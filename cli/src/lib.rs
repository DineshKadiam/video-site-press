// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ArgMatches;
use sys_locale::get_locale;

use hyper8_core::{Context, Language, SyncProtocol};

mod args;
mod build;
mod check;
mod deploy;
mod serve;
mod style;

use args::ArgMode;
use build::build;
use check::check;
use deploy::perform_deploy_blocking_cli;
use serve::serve;

fn handle(args: ArgMatches) -> ExitCode {
    let language = get_locale()
        .map(|locale| Language::from_code(&locale))
        .unwrap_or_else(|| Language::default());

    let translations = &language.translations;

    let site_dir = args.get_one::<PathBuf>("site-dir").unwrap();
    if !site_dir.exists() {
        eprintln!("Site directory {site_dir:?} does not exist, please create it manually (just an empty folder).");
        return ExitCode::FAILURE;
    }

    let custom_build_dir = args.get_one::<PathBuf>("build-dir");
    if let Some(build_dir) = custom_build_dir {
        if build_dir.parent().is_some_and(|dir| !dir.exists()) {
            eprintln!("The parent directory of the passed custom build directory {build_dir:?} does not exist, but this is not permitted. Please ensure it exists.");
            return ExitCode::FAILURE;
        }
    }

    let custom_cache_dir = args.get_one::<PathBuf>("cache-dir");
    if let Some(cache_dir) = custom_cache_dir {
        if cache_dir.parent().is_some_and(|dir| !dir.exists()) {
            eprintln!("The parent directory of the passed custom cache directory {cache_dir:?} does not exist, but this is not permitted. Please ensure it exists.");
            return ExitCode::FAILURE;
        }
    }

    let custom_worker_count = args.get_one::<usize>("workers").map(|count| *count);

    let context = Context::new(
        custom_build_dir,
        custom_cache_dir,
        custom_worker_count,
        site_dir
    );

    if args.get_flag("check-ftp") {
        check(context, SyncProtocol::Ftp, translations)
    } else if args.get_flag("check-ssh") {
        check(context, SyncProtocol::Ssh, translations)
    } else if args.get_flag("build") {
        build(args, context, translations)
    } else {
        serve(args, context)
    }
}

pub fn handle_cli_only() -> ExitCode {
    let args = args::parse(ArgMode::CliOnly);
    handle(args)
}

pub fn handle_desktop() -> Option<ExitCode> {
    let args = args::parse(ArgMode::Desktop);

    if args.args_present() {
        Some(handle(args))
    } else {
        None
    }
}

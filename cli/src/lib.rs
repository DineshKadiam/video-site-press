// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::process::ExitCode;

use anstream::eprintln;
use clap::ArgMatches;
use indoc::formatdoc;
use sys_locale::get_locale;

use hyper8_core::{Context, Language, SyncProtocol};

use crate::style::RED;

mod args;
mod build;
mod check;
mod prompt;
mod serve;
mod style;
mod sync;
mod sync_config;

use args::ArgMode;
use build::build;
use check::check;
use serve::serve;
use sync::sync;

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

    let non_interactive = args.get_flag("non-interactive");

    let delegate_check = |protocol| {
        check(
            &context,
            non_interactive,
            protocol,
            translations
        )
    };

    let delegate_sync = |protocol| {
        if context.build_dir.exists() {
            let simulate = args.get_flag("simulate");

            sync(
                &context,
                non_interactive,
                protocol,
                simulate,
                translations
            )
        } else {
            eprintln!("Not performing synchronization because the build directory does not exist. Maybe you wanted to use --build as well?");
            ExitCode::FAILURE
        }
    };

    fn print_ffmpeg_not_found() {
        let message = formatdoc!("
            {RED}ffmpeg could not be found{RED:#}

            This is likely a problem with your installation. If you built
            Hyper 8 yourself, make sure to install ffmpeg (including ffprobe)
            on your system. You can also compile hyper8 with --features
            use-bundled-ffmpeg and place an ffmpeg and ffprobe executable
            next to the hyper8 executable if you prefer not to install
            ffmpeg system-wide.
        ");

        eprintln!("{message}");
    }

    if args.get_flag("check-ftp") {
        delegate_check(SyncProtocol::Ftp)
    } else if args.get_flag("check-ssh") {
        delegate_check(SyncProtocol::Ssh)
    } else if args.get_flag("build") {
        let design_overlay = args.get_flag("design");

        if context.ffmpeg.is_none() {
            print_ffmpeg_not_found();
            return ExitCode::FAILURE;
        }

        build(
            args,
            &context,
            design_overlay,
            translations
        )
    } else if args.get_flag("sync-ftp") {
        delegate_sync(SyncProtocol::Ftp)
    } else if args.get_flag("sync-ssh") {
        delegate_sync(SyncProtocol::Ssh)
    } else {
        if context.ffmpeg.is_none() {
            print_ffmpeg_not_found();
            return ExitCode::FAILURE;
        }

        serve(args, &context)
    }
}

pub fn handle_cli_only() -> ExitCode {
    let args = args::parse(ArgMode::CliOnly);
    handle(args)
}

pub fn handle_desktop() -> Option<ExitCode> {
    let args = args::parse(ArgMode::Desktop);

    if args.args_present() {
        if args.contains_id("site-dir") {
            Some(handle(args))
        } else {
            eprintln!("The positional [site-dir] argument is mandatory when passing any command line arguments, you need to provide it.");
            Some(ExitCode::FAILURE)
        }
    } else {
        None
    }
}

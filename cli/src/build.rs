// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::process::ExitCode;
use std::sync::Arc;

use clap::ArgMatches;

use hyper8_core::{Context, SyncProtocol};
use hyper8_core::{perform_build, start_preview_server};
use hyper8_translations::Translations;

use crate::sync;

pub fn build(
    args: ArgMatches,
    context: &Arc<Context>,
    design_overlay: bool,
    translations: &Translations
) -> ExitCode {
    if let Err(err) = context.await_job_queue(translations) {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    if let Err(err) = perform_build(context, design_overlay) {
        eprintln!("Build failed:\n{err}");
        return ExitCode::FAILURE;
    }

    let delegate_sync = |protocol| {
        let non_interactive = args.get_flag("non-interactive");
        let simulate = args.get_flag("simulate");

        sync(
            context,
            non_interactive,
            protocol,
            simulate,
            translations
        )
    };

    if args.get_flag("sync-ftp") {
        delegate_sync(SyncProtocol::Ftp)
    } else if args.get_flag("sync-ssh") {
        delegate_sync(SyncProtocol::Ssh)
    } else if args.get_flag("preview") {
        let site = context.get_site();

        if site.clean_urls {
            let ip = args.get_one::<IpAddr>("ip");
            let port = args.get_one::<u16>("port");

            // Here we serve the preview through an actual http server so
            // that /foo/ gets resolved to /foo/index.html.
            let result = start_preview_server(&context.build_dir, ip, port);

            if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        } else {
            // We don't need an actively running server to preview a build
            // without clean urls, we can just open everything directly in
            // a browser.
            let local_file_url = context.build_dir.join("index.html");
            if webbrowser::open(&local_file_url.to_string_lossy()).is_err() {
                eprintln!("Could not open browser for previewing the site");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    } else {
        ExitCode::SUCCESS
    }
}

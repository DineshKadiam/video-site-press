// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::process::ExitCode;
use std::sync::Arc;

use clap::ArgMatches;

use hyper8_core::{Context, DeploymentProtocol};
use hyper8_core::{perform_build, start_preview_server};
use hyper8_translations::Translations;

use crate::perform_deploy_blocking_cli;

pub fn build(
    args: ArgMatches,
    context: Arc<Context>,
    translations: &Translations
) -> ExitCode {
    if let Err(err) = context.await_job_queue(translations) {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    if let Err(err) = perform_build(&context) {
        eprintln!("Build failed:\n{err}");
        return ExitCode::FAILURE;
    }

    if args.contains_id("deploy") {
        let site = context.get_site();

        let protocol = if args.get_flag("deploy-ftp") {
            DeploymentProtocol::Ftp
        } else /* if args.get_flag("deploy-ssh") */ {
            DeploymentProtocol::Ssh
        };

        let simulate = args.get_flag("simulate");

        match perform_deploy_blocking_cli(
            &context,
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

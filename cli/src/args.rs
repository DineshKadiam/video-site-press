// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::path::PathBuf;

use clap::{
    Arg,
    ArgAction,
    ArgGroup,
    ArgMatches,
    Command
};
use clap::value_parser;
use indoc::indoc;

use hyper8_version::VERSION_WITH_PATCH_AND_REVISION;

pub enum ArgMode {
    CliOnly,
    Desktop
}

pub fn parse(arg_mode: ArgMode) -> ArgMatches {
    let about;
    let after_help_text;
    let site_dir_required;
    let version;

    match arg_mode {
        ArgMode::CliOnly => {
            about = "A static site generator for video publishing (Command line only edition)";
            after_help_text = "Without specific flags (--build, etc.) the editor is started.";
            site_dir_required = true;
            version = format!("{VERSION_WITH_PATCH_AND_REVISION} (Command line only edition)");
        }
        ArgMode::Desktop => {
            about = "A static site generator for video publishing (Full desktop edition)";
            after_help_text = "To start the graphical launcher pass no arguments at all.";
            site_dir_required = false;
            version = format!("{VERSION_WITH_PATCH_AND_REVISION} (Full desktop edition)");
        }
    }

    let build = Arg::new("build")
        .action(ArgAction::SetTrue)
        .help("Directly build the site without starting the editor")
        .long("build")
        .short('b');

    let build_dir = Arg::new("build-dir")
        .help(indoc!("
            The path to which the files for the site are written

            WARNING: This directory is WIPED during the build process.
            Please pay very close attention where you point this to.

            (Default location: .hyper8_build/ inside the site directory)
        ").trim_end())
        .long("build-dir")
        .value_parser(value_parser!(PathBuf));

    let cache_dir = Arg::new("cache-dir")
        .help(indoc!("
            The path at which all computational results (computed metadata, resized images, temporary encoding artifacts) are stored.

            WARNING: This directory may be WIPED during the build process.
            Please pay very close attention where you point this to.

            (Default location: .hyper8_cache/ inside the site directory)
        ").trim_end())
        .long("cache-dir")
        .value_parser(value_parser!(PathBuf));

    let check = ArgGroup::new("check")
        .args(["check-ftp", "check-ssh"]);

    let check_ftp = Arg::new("check-ftp")
        .action(ArgAction::SetTrue)
        .help("Run a check to verify that the FTP/FTPS sync configuration and server setup is correct.")
        .help_heading("Sync options")
        .long("check-ftp");

    let check_ssh = Arg::new("check-ssh")
        .action(ArgAction::SetTrue)
        .help("Run a check to verify that the SSH sync configuration and server setup is correct.")
        .help_heading("Sync options")
        .long("check-ssh");

    let design = Arg::new("design")
        .action(ArgAction::SetTrue)
        .help("Include the design overlay on the site, allowing live exploration of different design settings.")
        .long("design")
        .requires("build");

    let ip = Arg::new("ip")
        .help("Manually set the ip address used by the editor or preview server (otherwise default to localhost)")
        .long("ip")
        .value_parser(value_parser!(IpAddr));

    let non_interactive = Arg::new("non-interactive")
        .action(ArgAction::SetTrue)
        .help(r#"Disable interactive command line prompts (sync confirmations, password prompts)."#)
        .long("non-interactive");

    let port = Arg::new("port")
        .help("Manually set the port used by the web editor (otherwise automatically choose an available port)")
        .long("port")
        .value_parser(value_parser!(u16));

    let preview = Arg::new("preview")
        .action(ArgAction::SetTrue)
        .help("Open the site in the browser after building. Use with --build.")
        .long("preview")
        .requires("build")
        .short('p');

    let simulate = Arg::new("simulate")
        .action(ArgAction::SetTrue)
        .help("Simulate synchronization by merely printing, but not executing, file synchronization actions. Use with --sync-ftp/--sync-ssh.")
        .help_heading("Sync options")
        .long("simulate")
        .requires("sync");

    let site_dir = Arg::new("site-dir")
        .help("The path at which the site resides (site manifest, videos, etc.)")
        .required(site_dir_required)
        .value_parser(value_parser!(PathBuf));

    let sync = ArgGroup::new("sync")
        .args(["sync-ftp", "sync-ssh"]);

    let sync_ftp = Arg::new("sync-ftp")
        .action(ArgAction::SetTrue)
        .help("Synchronize the build directory to the configured remote server over FTP/FTPS. Use with --build or on its own.")
        .help_heading("Sync options")
        .long("sync-ftp");

    let sync_ssh = Arg::new("sync-ssh")
        .action(ArgAction::SetTrue)
        .help("Synchronize the build directory to the configured remote server over SSH. Use with --build or on its own.")
        .help_heading("Sync options")
        .long("sync-ssh");

    let workers = Arg::new("workers")
        .help("Set the number of threads used for background processing (metadata computation, image processing, video/audio transcoding, etc.)")
        .long("workers")
        .value_parser(worker_count);

    let command = Command::new("Hyper 8 Video System")
        .about(about)
        .after_help(after_help_text)
        .arg(build)
        .arg(build_dir)
        .arg(cache_dir)
        .group(check)
        .arg(check_ftp)
        .arg(check_ssh)
        .arg(design)
        .arg(ip)
        .arg(non_interactive)
        .arg(port)
        .arg(preview)
        .arg(simulate)
        .arg(site_dir)
        .group(sync)
        .arg(sync_ftp)
        .arg(sync_ssh)
        .arg(workers)
        .next_line_help(true)
        .version(version);

    command.get_matches()
}

fn worker_count(string: &str) -> Result<usize, String> {
    match string.parse::<usize>() {
        Ok(number) => match number >= 2 {
            true => Ok(number),
            false => Err(String::from("At least 2 workers required"))
        }
        Err(err) => Err(err.to_string())
    }
}

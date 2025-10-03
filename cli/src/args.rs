// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
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
            after_help_text = None;
            site_dir_required = true;
            version = format!("{VERSION_WITH_PATCH_AND_REVISION} (Command line only edition)");
        }
        ArgMode::Desktop => {
            about = "A static site generator for video publishing (Full desktop edition)";
            after_help_text = Some("To start the graphical launcher pass no arguments at all");
            site_dir_required = false;
            version = format!("{VERSION_WITH_PATCH_AND_REVISION} (Full desktop edition)");
        }
    }

    let build = Arg::new("build")
        .action(ArgAction::SetTrue)
        .help("Directly builds the site without starting the editor")
        .long("build")
        .short('b');

    let build_dir = Arg::new("build-dir")
        .help(indoc!("
            The path to which the deployable, built site is written

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

    let deploy = Arg::new("deploy")
        .action(ArgAction::SetTrue)
        .help("Directly deploys the site - only applicable in combination with --build")
        .long("deploy")
        .short('d');

    let ip = Arg::new("ip")
        .help("Manually sets the ip address used by the editor or preview server (otherwise defaults to localhost)")
        .long("ip")
        .value_parser(value_parser!(IpAddr));

    let port = Arg::new("port")
        .help("Manually sets the port used by the web editor (otherwise chooses an available port on its own)")
        .long("port")
        .value_parser(value_parser!(u16));

    let preview = Arg::new("preview")
        .action(ArgAction::SetTrue)
        .help("Use in conjunction with --build to immediately open the site in the browser after the build is complete")
        .long("preview")
        .short('p');

    let site_dir = Arg::new("site-dir")
        .help("The path at which the site resides (user videos and metadata)")
        .required(site_dir_required)
        .value_parser(value_parser!(PathBuf));

    let workers = Arg::new("workers")
        .help("The number of threads to run for background processing (metadata computation, image processing, video/audio transcoding, etc.)")
        .long("workers")
        .value_parser(worker_count);

    let mut command = Command::new("Hyper 8 Video System")
        .about(about)
        .arg(build)
        .arg(build_dir)
        .arg(cache_dir)
        .arg(deploy)
        .arg(ip)
        .arg(port)
        .arg(preview)
        .arg(site_dir)
        .arg(workers)
        .next_line_help(true)
        .version(version);

    if let Some(after_help_text) = after_help_text {
        command = command.after_help(after_help_text);
    }

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

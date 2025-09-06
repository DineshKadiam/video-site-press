// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
use clap::value_parser;

use hyper8_version::VERSION_WITH_PATCH_AND_REVISION;

pub fn parse() -> ArgMatches {
    let build = Arg::new("build")
        .action(ArgAction::SetTrue)
        .help("Directly builds the site without starting the editor")
        .long("build")
        .short('b');

    let build_dir = Arg::new("build-dir")
        .help("The path to which the deployable, built site is written")
        .long("build-dir")
        .value_parser(value_parser!(PathBuf));

    let cache_dir = Arg::new("cache-dir")
        .help("The path at which all computational results are stored (computed metadata, images and videos)")
        .long("cache-dir")
        .value_parser(value_parser!(PathBuf));

    let deploy = Arg::new("deploy")
        .action(ArgAction::SetTrue)
        .help("Directly deploys the site - only applicable in combination with --build")
        .long("deploy")
        .short('d');

    let ip = Arg::new("ip")
        .help("Manually sets the ip address used by the web editor (otherwise defaults to localhost)")
        .long("ip")
        .value_parser(value_parser!(IpAddr));

    let port = Arg::new("port")
        .help("Manually sets the port used by the web editor (otherwise hyper8 chooses an available port on its own)")
        .long("port")
        .value_parser(value_parser!(u16));

    let preview = Arg::new("preview")
        .action(ArgAction::SetTrue)
        .help("Use in conjunction with --build to immediately open the site in the browser after the build is complete")
        .long("preview")
        .short('p');

    let site_dir = Arg::new("site-dir")
        .help("The path at which the site resides (user videos and metadata)")
        .required(true)
        .value_parser(value_parser!(PathBuf));

    let workers = Arg::new("workers")
        .help("The number of threads to run for background processing (metadata computation, image processing, video/audio transcoding, etc.)")
        .long("workers")
        .value_parser(worker_count);

    let command = Command::new("Hyper 8 Video System")
        .version(VERSION_WITH_PATCH_AND_REVISION)
        .about("A static site generator for video publishing")
        .arg(build)
        .arg(build_dir)
        .arg(cache_dir)
        .arg(deploy)
        .arg(ip)
        .arg(port)
        .arg(preview)
        .arg(site_dir)
        .arg(workers);

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

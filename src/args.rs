// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;

fn worker_count(string: &str) -> Result<usize, String> {
    match string.parse::<usize>() {
        Ok(number) => match number >= 2 {
            true => Ok(number),
            false => Err(String::from("At least 2 workers required"))
        }
        Err(err) => Err(err.to_string())
    }
}

#[derive(Debug, Parser)]
#[clap(display_name = "Hyper 8 Video System", version = concat!(env!("HYPER8_VERSION_DETAILED"), " (", env!("HYPER8_REVISION"), ")"))]
pub struct Args {
    /// Directly builds the site without starting the editor
    #[clap(long = "build", short = 'b')]
    pub build: bool,

    /// The path to which the deployable, built site is written
    #[clap(long = "build-dir")]
    pub build_dir: Option<PathBuf>,

    /// The path at which all computational results are stored (computed metadata, images and videos)
    #[clap(long = "cache-dir")]
    pub cache_dir: Option<PathBuf>,

    /// Directly deploys the site - only applicable in combination with --build [HEADS UP: Incomplete implementation, for now use the interactive editor]
    #[clap(long)]
    pub deploy: bool,

    /// Override the language displayed in the editor (e.g. 'en', 'de')
    #[clap(long = "editor-language")]
    pub editor_language: Option<String>,

    /// Manually sets the ip address used by the web editor (otherwise defaults to localhost)
    #[clap(long = "ip")]
    pub ip: Option<IpAddr>,

    /// Manually sets the port used by the web editor (otherwise hyper8 chooses an available port on its own)
    #[clap(long = "port")]
    pub port: Option<u16>,

    /// Use in conjunction with --build to immediately open the site in the browser after the build is complete
    #[clap(long = "preview", short = 'p')]
    pub preview: bool,

    /// The path at which the site resides (user videos and metadata)
    pub site_dir: PathBuf,

    /// The number of threads doing background processing (video transcoding mostly)
    /// in the background
    #[clap(default_value_t = 2, long, value_parser=worker_count)]
    pub workers: usize
}

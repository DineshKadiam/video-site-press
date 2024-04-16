use clap::Parser;
use std::path::PathBuf;

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
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    /// Directly builds the site without starting the authoring server
    #[clap(long)]
    pub build: bool,

    /// The path to which the deployable, built site is written
    #[clap(long = "build-dir")]
    pub build_dir: Option<PathBuf>,

    /// The path at which all computational results are stored (computed metadata, images and videos)
    #[clap(long = "cache-dir")]
    pub cache_dir: Option<PathBuf>,

    /// Directly deploys the site - only applicable in combination with --build
    #[clap(long)]
    pub deploy: bool,

    /// Override the language displayed in the editor (e.g. 'en', 'de')
    #[clap(long = "editor-language")]
    pub editor_language: Option<String>,

    /// Manually sets the port used by the web editor (otherwise hyper8 chooses an available port on its own)
    #[clap(long = "port")]
    pub port: Option<u16>,

    /// The path at which the site resides (user videos and metadata)
    pub site_dir: PathBuf,

    /// The number of threads doing background processing (video transcoding mostly)
    /// in the background
    #[clap(default_value_t = 2, long, value_parser=worker_count)]
    pub workers: usize
}

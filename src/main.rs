use std::sync::{Arc, RwLock};

use clap::Parser;

mod args;
mod browser_support;
mod build;
mod cache;
mod collection;
mod context;
mod deploy;
mod editor;
mod hyper_dir;
mod image_processor;
mod playlist;
mod poster;
mod processing;
mod site;
mod theme;
mod transcode;
mod video;
mod video_format;
mod util;
mod watcher;
mod workers;

use args::Args;
use browser_support::BrowserSupport;
use cache::{Cache, CacheItem, FileMeta, PlaylistInternals, PosterAssetsCached, VideoMetaCached};
use collection::Collection;
use context::{Context, ContextPath, GetAnyResult};
use hyper_dir::HyperDir;
use image_processor::{ImageProcessor, ResizeMode};
use playlist::Playlist;
use poster::{PosterAsset, PosterAssets, PosterFile};
use processing::{Job, JobKind, Processing};
use site::{Site, SiteContent};
use theme::Theme;
use util::Field;
use video::{Video, VideoFile, VideoMeta};
use video_format::{Codec, Container, EncodingApproach, VideoFormat};

fn main() {
    let args: Args = Args::parse();

    if !args.site_dir.exists() {
        eprintln!(
            "Site directory {:?} does not exist, please create it manually (just an empty folder).",
            args.site_dir
        );
        return
    }

    let site_dir = args.site_dir.canonicalize().unwrap();

    let build_dir = args.build_dir
        .as_ref()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| site_dir.join(".hyper8_build"));

    // We would/could theoretically create this later (on build), but
    // as actix-files does not successfully register the /preview/
    // static file hosting route if the directory does not exist on
    // startup, we create it here already.
    util::ensure_dir(&build_dir);

    let cache_dir = args.cache_dir
        .as_ref()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| site_dir.join(".hyper8_cache"));

    util::ensure_dir(&cache_dir);

    let cache = RwLock::new(Cache::retrieve(&cache_dir));

    let context = Context::new(
        build_dir,
        cache,
        cache_dir,
        Arc::new(Site::new()), // TODO: Clean this up, super hacky
        site_dir.clone()
    );

    *context.site.write().unwrap() = Site::read_dir(&context, &site_dir);

    let context = Arc::new(context);

    if args.build {
        // TODO: In --build mode we still need to process all assets (video encoding!)
        //       before we can build the website, this is fully missing right now.

        build::perform_build(&context);
        
        if args.deploy {
            deploy::perform_deploy(&context);
        }
    } else {
        context.compute_job_queue();

        println!("Using site directory: {}", context.site_dir.display());
        println!("Serving at: http://localhost:8081");

        workers::start_threads(args.workers, &context);

        watcher::watch(Arc::clone(&context));

        editor::server::start(context);
    }
}
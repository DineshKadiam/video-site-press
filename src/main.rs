use std::sync::{Arc, Mutex};

use clap::Parser;
use sys_locale::get_locale;

mod args;
mod banner;
mod browser_support;
mod build;
mod cache;
mod collection;
mod context;
mod deploy;
mod editor;
mod font;
mod hyper_dir;
mod icons;
mod image_processor;
mod language;
mod playlist;
mod poster;
mod processing;
mod site;
mod subtitles;
mod theme;
mod transcode;
mod translations;
mod video;
mod video_format;
mod util;
mod watcher;
mod workers;

use args::Args;
use banner::Banner;
use cache::{Cache, CacheItem, FileMeta, PosterAssetsCached, VideoMetaCached};
use collection::Collection;
use context::{Context, ContextPath, GetAnyResult};
use editor::EditorConfig;
use font::Font;
use hyper_dir::HyperDir;
use image_processor::{ImageProcessor, ResizeMode};
use language::Language;
use playlist::Playlist;
use poster::{PosterAsset, PosterAssets, PosterFile};
use processing::{Job, JobKind, Processing, Progress};
use site::{AspectRatio, BaseUrl, Site, SiteContent};
use subtitles::SubtitleFile;
use theme::{Theme, ThemeKind};
use translations::Translations;
use util::Field;
use video::{Video, VideoFile, VideoMeta, VideoParent};
use video_format::{Codec, EncodingApproach, VideoFormat};

fn main() {
    let args: Args = Args::parse();
    let editor_config = EditorConfig::retrieve();

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

    let cache = Mutex::new(Cache::retrieve(&cache_dir));

    let editor_language = args.editor_language
        .unwrap_or_else(|| {
            match &editor_config {
                Some(editor_config) => editor_config.language_code.clone(),
                None => get_locale()
                    .map(|locale| locale.split('-').next().unwrap().to_string())
                    .unwrap_or_else(|| String::from("en"))
            }
        });

    let context = Context::new(
        build_dir,
        cache,
        cache_dir,
        &editor_language,
        &editor_config.as_ref().map(|config| config.theme_key.as_str()).unwrap_or("adaptive"),
        Arc::new(Site::new()),
        site_dir.clone()
    );

    // TODO: Would be great if we could somehow resolve the conundrum that
    //       context depends on site and site depends on context, which
    //       forces us to initialize context with a dummy site and replace it here.
    context.replace_site(Site::read_dir(&context, &site_dir));

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

        workers::start_threads(args.workers, &context);

        watcher::watch(Arc::clone(&context));

        editor::server::start(context, &args.port);
    }
}
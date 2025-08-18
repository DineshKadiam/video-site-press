// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use clap::Parser;
use sys_locale::get_locale;

mod args;
mod aspect_ratio;
mod audio_file;
mod audio_format;
mod audio_meta;
mod banner;
mod browser_support;
mod build;
mod cache;
mod collection;
mod context;
mod deploy;
mod editor;
mod file_meta;
mod font;
mod hyper_dir;
mod icons;
mod image_processor;
mod inherited_options;
mod jobs;
mod language;
mod link;
mod lqip;
mod manifest;
mod markdown_subset;
mod platform_integration;
mod playlist;
mod poster_assets;
mod poster_file;
mod poster_meta;
mod preview_server;
mod site;
mod site_path;
mod site_url;
mod subtitles;
mod theme;
mod transcode;
mod translations;
mod video;
mod video_file;
mod video_format;
mod video_meta;
mod video_order;
mod util;
mod watcher;
mod workers;

use args::Args;
use aspect_ratio::AspectRatio;
use audio_file::AudioFile;
use audio_format::AudioFormat;
use audio_meta::AudioMeta;
use banner::Banner;
use cache::{AudioMetaCached, Cache, PosterAssetsCached, PosterSourceCached, VideoMetaCached};
use collection::Collection;
use context::{CollectionUpdate, Context, GetAnyResult, PlaylistUpdate, SiteUpdate, VideoUpdate};
use deploy::{DeploymentState, FtpConfig, SshConfig};
use editor::{EditorConfig, TreeViewOrder};
use file_meta::FileMeta;
use font::Font;
use hyper_dir::HyperDir;
use image_processor::{ImageProcessor, ResizeMode};
use inherited_options::InheritedOptions;
use jobs::{ActiveJobs, Job, JobKind, JobProgress};
use language::Language;
use link::Link;
use lqip::Lqip;
use markdown_subset::MarkdownSubset;
use platform_integration::{PlatformIntegration, PlatformIntegrationImage, PlatformIntegrationMeta};
use playlist::Playlist;
use poster_assets::{PosterAsset, PosterAssets};
use poster_file::PosterFile;
use poster_meta::PosterMeta;
use site::{Site, SiteContent};
use site_path::SitePath;
use site_url::SiteUrl;
use subtitles::SubtitleFile;
use theme::{Theme, ThemeKind};
use transcode::FFmpegEncodingStats;
use translations::Translations;
use util::Field;
use video::Video;
use video_file::VideoFile;
use video_format::{Codec, EncodingApproach, VideoFormat};
use video_meta::VideoMeta;
use video_order::VideoOrder;

fn main() -> ExitCode {
    env_logger::init();

    let args: Args = Args::parse();
    let editor_config = EditorConfig::retrieve();

    if !args.site_dir.exists() {
        eprintln!(
            "Site directory {:?} does not exist, please create it manually (just an empty folder).",
            args.site_dir
        );
        return ExitCode::FAILURE;
    }

    // This compiles to fs::canonicalize(...) on all platforms but windows,
    // on windows it instead tries to supply a canonicalized path that uses
    // the regular form (e.g. "C:\\Foo") instead of the UNC form ("\\?C:\\Foo")
    // when possible (for background see https://crates.io/crates/dunce).
    let site_dir = dunce::canonicalize(args.site_dir).unwrap();

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

    let editor_tree_view_order = match &editor_config {
        Some(editor_config) => editor_config.tree_view_order,
        None => TreeViewOrder::Default
    };

    let context = Context::new(
        build_dir,
        cache,
        cache_dir,
        &editor_language,
        editor_config.as_ref().map(|config| config.theme_key.as_str()).unwrap_or("adaptive"),
        editor_tree_view_order,
        Arc::new(Site::new()),
        site_dir.clone()
    );

    // TODO: Would be great if we could somehow resolve the conundrum that
    //       context depends on site and site depends on context, which
    //       forces us to initialize context with a dummy site and replace it here.
    context.replace_site(Site::read_dir(&context, &site_dir));

    let context = Arc::new(context);

    context.compute_job_queue();
    workers::start_threads(args.workers, &context);

    if args.build {
        context.await_job_queue();

        build::perform_build(&context);

        if args.preview {
            let site = context.get_site();

            let result = if site.clean_urls {
                // Here we serve the preview through an actual http server so
                // that /foo/ gets resolved to /foo/index.html.
                preview_server::start(&context.build_dir, &args.ip, &args.port)
            } else {
                // TODO: Uncomment below code and remove the currently used
                // line (preview_server::start(...)) below it as soon as we have
                // switched back to using exclusively non-ESM scripts on the
                // client-side.

                // // We don't need an actively running server to preview a build
                // // without clean urls, we can just open everything directly in
                // // a browser.
                // let local_file_url = context.build_dir.join("index.html");
                // if webbrowser::open(&local_file_url.to_string_lossy()).is_err() {
                //     eprintln!("Could not open browser for previewing the site");
                //     return ExitCode::FAILURE
                // }

                preview_server::start(&context.build_dir, &args.ip, &args.port)
            };

            if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        } else if args.deploy {
            let site = context.get_site();
            let language = context.get_editor_language();

            // TODO: Deployment is async now (this call immediately returns).
            //       Need to switch to a blocking version that also reports output to the console (?)
            match deploy::perform_deploy(&context, &site, &language.translations) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
            }
        } else {
            ExitCode::SUCCESS
        }
    } else {
        println!("Using site directory: {}", context.site_dir.display());

        watcher::watch(Arc::clone(&context));
        let result = editor::server::start(context, &args.ip, &args.port);

        if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
    }
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

mod aspect_ratio;
mod audio_file;
mod audio_format;
mod audio_meta;
mod banner;
mod browser_support;
mod build;
mod cache;
mod collection;
mod container;
mod container_order;
mod context;
mod file_meta;
mod ffmpeg;
mod font;
mod hyper_dir;
mod image_processor;
mod inherited_options;
mod jobs;
mod language;
mod link;
mod lqip;
mod manifest;
mod markdown_subset;
mod media_usage;
mod percent;
mod platform_integration;
mod playlist;
mod poster_assets;
mod poster_file;
mod poster_meta;
mod preview_server;
mod server_defaults;
mod site;
mod site_path;
mod site_url;
mod sort;
mod subtitle_file;
mod sync;
mod theme;
mod video;
mod video_file;
mod video_format;
mod video_meta;
mod video_order;

pub mod util;
pub mod watcher;

use audio_meta::AudioMeta;
use cache::{AudioMetaCached, Cache, PosterAssetsCached, PosterSourceCached, VideoMetaCached};
use ffmpeg::FFmpeg;
use image_processor::{ImageProcessor, ResizeMode};
use inherited_options::InheritedOptions;
use jobs::{CompletedJob, JobError, QueuedJob, Worker};
use jobs::start_workers;
use lqip::Lqip;
use markdown_subset::MarkdownSubset;
use platform_integration::{PlatformIntegrationImage, PlatformIntegrationMeta};
use poster_assets::{PosterAsset, PosterAssets};
use poster_meta::PosterMeta;
use sort::SortDirection;
use sync::{SyncAction, SyncError};
use video_format::{Codec, EncodingApproach};
use video_meta::VideoMeta;

pub use aspect_ratio::AspectRatio;
pub use audio_file::AudioFile;
pub use audio_format::AudioFormat;
pub use banner::Banner;
pub use browser_support::BrowserSupport;
pub use build::perform_build;
pub use collection::Collection;
pub use container::Container;
pub use container_order::ContainerOrder;
pub use context::{
    CollectionUpdate,
    Context,
    GetAnyResult,
    PlaylistUpdate,
    SiteUpdate,
    VideoParent,
    VideoUpdate
};
pub use file_meta::FileMeta;
pub use font::Font;
pub use hyper_dir::HyperDir;
pub use jobs::{JobDesc, JobProgress};
pub use language::AVAILABLE_LANGUAGES;
pub use language::Language;
pub use link::Link;
pub use manifest::MAX_SYNOPSIS_CHARS;
pub use media_usage::MediaUsage;
pub use percent::Percent;
pub use platform_integration::PlatformIntegration;
pub use playlist::Playlist;
pub use poster_file::PosterFile;
pub use preview_server::start as start_preview_server;
pub use server_defaults::DEFAULT_IP;
pub use server_defaults::{default_ports_display, default_ports_iter};
pub use site::{Site, SiteContent};
pub use site_path::SitePath;
pub use site_url::SiteUrl;
pub use sort::{Asc, Comparison, Desc};
pub use subtitle_file::{
    SubtitleFile,
    SubtitleFormat,
    Subtitles
};
pub use sync::{
    FtpConfig,
    SshConfig,
    SyncConfigCheckStep,
    SyncEvent,
    SyncEventActionKind,
    SyncEventKind,
    SyncProtocol,
    SyncState
};
pub use sync::{check_sync_config, start_sync};
pub use theme::SPLASH_SVG;
pub use theme::{Theme, ThemeKind};
pub use util::InfallibleResult;
pub use video::Video;
pub use video_file::VideoFile;
pub use video_format::VideoFormat;
pub use video_order::VideoOrder;

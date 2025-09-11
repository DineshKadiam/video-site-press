// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

mod aspect_ratio;
mod audio_file;
mod audio_format;
mod audio_meta;
mod banner;
mod build;
mod cache;
mod collection;
mod container;
mod container_order;
mod context;
mod deploy;
mod file_meta;
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
mod platform_integration;
mod playlist;
mod poster_assets;
mod poster_file;
mod poster_meta;
mod preview_server;
mod site;
mod site_path;
mod site_url;
mod sort;
mod subtitles;
mod theme;
mod video;
mod video_file;
mod video_format;
mod video_meta;
mod video_order;

pub mod browser_support;
pub mod util;
pub mod watcher;

use audio_meta::AudioMeta;
use cache::{AudioMetaCached, Cache, PosterAssetsCached, PosterSourceCached, VideoMetaCached};
use deploy::{FtpConfig, SshConfig};
use image_processor::{ImageProcessor, ResizeMode};
use inherited_options::InheritedOptions;
use jobs::{CompletedJob, QueuedJob, Worker};
use jobs::start_workers;
use lqip::Lqip;
use markdown_subset::MarkdownSubset;
use platform_integration::{PlatformIntegrationImage, PlatformIntegrationMeta};
use poster_assets::{PosterAsset, PosterAssets};
use poster_meta::PosterMeta;
use sort::SortDirection;
use video_format::{Codec, EncodingApproach};
use video_meta::VideoMeta;

pub use aspect_ratio::AspectRatio;
pub use audio_file::AudioFile;
pub use audio_format::AudioFormat;
pub use banner::Banner;
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
pub use deploy::DeploymentState;
// TODO: Rename "config_probe" to something clearer? (e.g. test_deployment_config or such)
pub use deploy::{config_probe, start_deploy};
pub use file_meta::FileMeta;
pub use font::Font;
pub use hyper_dir::HyperDir;
pub use jobs::{JobDesc, JobProgress};
pub use language::AVAILABLE_LANGUAGES;
pub use language::Language;
pub use link::Link;
pub use manifest::MAX_SYNOPSIS_CHARS;
pub use platform_integration::PlatformIntegration;
pub use playlist::Playlist;
pub use poster_file::PosterFile;
pub use preview_server::start as start_preview_server;
pub use site::{Site, SiteContent};
pub use site_path::SitePath;
pub use site_url::SiteUrl;
pub use sort::{Asc, Comparison, Desc};
pub use subtitles::SubtitleFile;
pub use theme::{Theme, ThemeKind};
pub use video::Video;
pub use video_file::VideoFile;
pub use video_format::VideoFormat;
pub use video_order::VideoOrder;

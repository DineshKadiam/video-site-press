// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::{
    Context,
    Collection,
    Font,
    GetAnyResult,
    HyperDir,
    HyperUrl,
    InheritedOptions,
    Language,
    Playlist,
    Theme,
    Video
};
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::supported_font_extension;

const SITE_OPTIONS: &[&str] = &[
    "base_url",
    "disable_clean_urls",
    "deploy_config",
    "font",
    "language",
    "poster_aspect",
    "theme"
];

#[derive(Clone, Debug)]
pub enum AspectRatio {
    Custom {
        representation: String,
        value: f32
    },
    Default
}

#[derive(Clone, Debug)]
pub struct DeployConfig {
    pub path: Option<String>,
    pub server: Option<String>,
    pub user: Option<String>
}

#[derive(Clone, Debug)]
pub struct Site {
    pub base_url: Option<HyperUrl>,
    pub clean_urls: bool,
    pub content: SiteContent,
    pub deploy_config: DeployConfig,
    pub errors: Vec<String>,
    pub font: Font,
    pub language: Language,
    pub poster_aspect: AspectRatio,
    pub theme: Theme
}

#[derive(Clone, Debug)]
pub enum SiteContent {
    Collection(Arc<Collection>),
    Empty,
    Invalid,
    Playlist(Arc<Playlist>),
    Video(Arc<Video>)
}

impl AspectRatio {
    pub fn from_representation(representation: &str) -> Result<AspectRatio, String> {
        if let Ok(ratio) = representation.parse::<f32>() {
            let aspect_ratio = AspectRatio::Custom {
                representation: representation.to_owned(),
                value: ratio
            };

            return Ok(aspect_ratio);
        }

        let mut parts = representation.split(|c| c == '/' || c == ':');
        if let (Some(dividend), Some(divisor), None) = (parts.next(), parts.next(), parts.next()) {
            if let (Ok(dividend), Ok(divisor)) = (dividend.parse::<f32>(), divisor.parse::<f32>()) {
                let aspect_ratio = AspectRatio::Custom {
                    representation: representation.to_owned(),
                    value: dividend / divisor
                };

                return Ok(aspect_ratio);
            }
        }

        let error = format!("'{representation}' is not understood, allowed aspect formats are e.g. '1.78', '16:9' or '16/9')");
        Err(error)
    }

    pub fn numeric(&self) -> f32 {
        match self {
            AspectRatio::Default => 16.0/9.0,
            AspectRatio::Custom { value, .. } => *value
        }
    }
}

impl DeployConfig {
    pub fn complete(&self) -> bool {
        self.path.is_some() && self.server.is_some() && self.user.is_some()
    }

    pub fn new() -> DeployConfig {
        DeployConfig {
            path: None,
            server: None,
            user: None
        }
    }
}

impl Site {
    fn apply_manifest(&mut self, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return;
            }
        };

        for element in document.elements() {
            match element.key() {
                "base_url" => 'base_url: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match HyperUrl::parse(value) {
                                    Ok(url) => {
                                        self.base_url = Some(url);
                                    }
                                    Err(err) => {
                                        let message = format!("The supplied url seems to be malformed ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'base_url;
                        }
                    }

                    let message = "base_url needs to be provided as a field with a value, e.g.: 'base_url: https://example.com'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "deploy_config" => 'deploy_config: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(attributes) = field.attributes() {
                            for attribute in attributes {
                                match attribute.key() {
                                    "path" => {
                                        if let Some(value) = attribute.value() {
                                            self.deploy_config.path = Some(value.to_string());
                                        }
                                    }
                                    "server" => {
                                        if let Some(value) = attribute.value() {
                                            self.deploy_config.server = Some(value.to_string());
                                        }
                                    }
                                    "user" => {
                                        if let Some(value) = attribute.value() {
                                            self.deploy_config.user = Some(value.to_string());
                                        }
                                    }
                                    _ => {
                                        let message = "The key/name of this attribute was not recognized, only 'path', 'server' and 'user' are recognized inside a deploy_config field";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'deploy_config;
                        }
                    }

                    let message = "deploy_config needs to be provided as a field with attributes, e.g.:\n\ndeploy_config:\npath = /var/www/example/\nserver = example.com\nuser = alice";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "disable_clean_urls" => {
                    if element.is_flag() {
                        self.clean_urls = false;
                    } else {
                        let message = "disable_clean_urls needs to be provided as a flag, that is, exactly as 'disable_clean_urls' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.errors.push(error);
                    }
                }
                "font" => 'font: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                let absolute_path = manifest_path.parent().unwrap().join(value);
                                if absolute_path.exists() {
                                    if supported_font_extension(value) {
                                        self.font = Font::Custom(value.to_string());
                                    } else {
                                        let message = "The provided font file extension is not supported, only woff and woff2 are";
                                        let error = element_error(element, manifest_path, message);
                                        self.errors.push(error);
                                    }
                                } else {
                                    let message = "The provided font file name is invalid - the file was not found";
                                    let error = element_error(element, manifest_path, message);
                                    self.errors.push(error);
                                }
                            }

                            break 'font;
                        }
                    }

                    let message = "font needs to be provided as a field with a value specifying the woff or woff2 font file name, e.g.: 'font: custom_sans.woff2'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "language" => 'language: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                self.language = Language::from_code(value);
                            }

                            break 'language;
                        }
                    }

                    let message = "language needs to be provided as a field with a value, e.g.: 'language: de'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "poster_aspect" => 'poster_aspect: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match AspectRatio::from_representation(value) {
                                    Ok(aspect_ratio) => {
                                        self.poster_aspect = aspect_ratio;
                                    }
                                    Err(err) => {
                                        let message = format!("The supplied aspect ratio was not recognized ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'poster_aspect;
                        }
                    }

                    let message = "poster_aspect needs to be provided as a field with a value, e.g.: 'poster_aspect: 16:9'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "theme" => 'theme: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match Theme::from_key(value) {
                                    Ok(theme) => {
                                        self.theme = theme;
                                    }
                                    Err(err) => {
                                        let message = format!("The supplied theme key was no recognized ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'theme;
                        }
                    }

                    let message = "theme needs to be provided as a field with a value, e.g.: 'theme: light_warm'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                other => {
                    let message = not_supported_error("site.eno", other, &[SITE_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.errors.push(error);
                }
            }
        }
    }

    /// Traverses the site tree to determine if any video on the site has
    /// copy link enabled. This information is consecutively used to
    /// determine whether we need to write clipboard css/js assets to the
    /// build.
    pub fn copy_link_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.copy_link_requested(),
            SiteContent::Playlist(playlist) => playlist.copy_link_requested(),
            SiteContent::Video(video) => video.copy_link(),
            SiteContent::Empty |
            SiteContent::Invalid => false
        }
    }

    /// Traverses the site tree to determine if any video on the site has
    /// embeds enabled. This information is consecutively used to determine
    /// whether we need to write embed css/js assets to the build.
    pub fn embeds_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.embeds_requested(),
            SiteContent::Playlist(playlist) => playlist.embeds_requested(),
            SiteContent::Video(video) => video.embedding(),
            SiteContent::Empty |
            SiteContent::Invalid => false
        }
    }

    /// Returns a collection, playlist or video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        if path.is_empty() {
            match &self.content {
                SiteContent::Collection(collection) => GetAnyResult::Collection(Arc::clone(collection)),
                SiteContent::Playlist(playlist) => GetAnyResult::Playlist(Arc::clone(playlist)),
                SiteContent::Video(video) => GetAnyResult::Video(Arc::clone(video)),
                _ => GetAnyResult::None
            }
        } else {
            match &self.content {
                SiteContent::Collection(collection) => collection.get_any(path),
                SiteContent::Playlist(playlist) => playlist.get_any(path),
                _ => GetAnyResult::None
            }
        }
    }

    /// Returns the collection at the given path.
    pub fn get_collection(&self, path: &str) -> Option<Arc<Collection>> {
        if let SiteContent::Collection(collection) = &self.content {
            if path.is_empty() {
                return Some(Arc::clone(collection));
            } else {
                return collection.get_collection(path);
            }
        }

        None
    }

    /// Returns the playlist at the given path.
    pub fn get_playlist(&self, path: &str) -> Option<Arc<Playlist>> {
        if path.is_empty() {
            if let SiteContent::Playlist(playlist) = &self.content {
                return Some(Arc::clone(playlist));
            }
        } else if let SiteContent::Collection(collection) = &self.content {
            return collection.get_playlist(path)
        }

        None
    }

    /// Returns the video at the given path.
    pub fn get_video(&self, path: &str) -> Option<Arc<Video>> {
        if path.is_empty() {
            if let SiteContent::Video(video) = &self.content {
                return Some(Arc::clone(video));
            }
        } else if let SiteContent::Playlist(playlist) = &self.content {
            return playlist.get_video(path)
        } else if let SiteContent::Collection(collection) = &self.content {
            return collection.get_video(path)
        }

        None
    }

    /// When we construct site-internal linking urls, we always
    /// append an index suffix. For instance we might build this:
    /// root_prefix ("../") + permalink ("foo") + index_suffix ("/") = "../foo/"
    /// If clean_urls is disabled however, we always append an index_suffix "/index.html",
    /// so that above example would result in "../foo/index.html".
    pub fn index_suffix(&self) -> &str {
        match self.clean_urls {
            true => "/",
            false => "/index.html"
        }
    }

    pub fn new() -> Site {
        Site {
            base_url: None,
            content: SiteContent::Empty,
            clean_urls: true,
            deploy_config: DeployConfig::new(),
            errors: Vec::new(),
            font: Font::Default,
            language: Language::default(),
            poster_aspect: AspectRatio::Default,
            theme: Theme::default()
        }
    }

    pub fn read_dir(context: &Context, dir: &Path) -> Site {
        let hyper_dir = HyperDir::read(dir);

        let mut site = Site::new();

        if hyper_dir.site_manifest.is_some() {
            site.apply_manifest(hyper_dir.site_manifest.as_deref().unwrap());
        }

        let default_options = InheritedOptions::default();

        if hyper_dir.collection_manifest.is_some() {
            if hyper_dir.playlist_manifest.is_some() || hyper_dir.video_manifest.is_some() {
                let error = "Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.".to_string();
                site.errors.push(error);
                site.content = SiteContent::Invalid;
            } else {
                let collection = Collection::read_dir(context, &hyper_dir, default_options, &site);
                site.content = SiteContent::Collection(collection);
            }
        } else if hyper_dir.playlist_manifest.is_some() {
            if hyper_dir.video_manifest.is_some() {
                let error = "Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.".to_string();
                site.errors.push(error);
                site.content = SiteContent::Invalid;
            } else {
                let playlist = Playlist::read_dir(context, &hyper_dir, default_options, &site);
                site.content = SiteContent::Playlist(playlist);
            }
        } else if hyper_dir.video_manifest.is_some() {
            let video = Video::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Video(video);
        } else if !hyper_dir.dirs.is_empty() {
            let collection = Collection::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Collection(collection);
        } else if !hyper_dir.video_files.is_empty() {
            let video = Video::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Video(video);
        }

        site
    }

    /// Returns storage used in bytes (only takes videos into account for now)
    pub fn storage_usage(&self) -> u64 {
        fn video_size(video: &Video) -> u64 {
            video.versions.iter().map(|version| version.file_meta.size).sum()
        }

        fn playlist_size(playlist: &Playlist) -> u64 {
            playlist.videos.iter().map(|video| video_size(video)).sum()
        }

        match &self.content {
            SiteContent::Collection(collection) => {
                fn collection_size(collection: &Collection) -> u64 {
                    let mut size = 0;

                    for playlist in &collection.playlists {
                        size += playlist_size(playlist)
                    }

                    for subcollection in &collection.subcollections {
                        size += collection_size(subcollection);
                    }

                    for video in &collection.videos {
                        size += video_size(video)
                    }

                    size
                }

                collection_size(collection)
            }
            SiteContent::Playlist(playlist) => playlist_size(playlist),
            SiteContent::Video(video) => video_size(video),
            _ => 0
        }
    }

    pub fn title(&self) -> &str {
        match &self.content {
            SiteContent::Collection(collection) => collection.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Playlist(playlist) => playlist.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Video(video) => video.title.as_deref().unwrap_or("Hyper 8"),
            _ => "Hyper 8"
        }
    }

    /// Creates a relative link to go 0 (= refresh current page) or
    /// more levels upwards towards the site root, taking into account
    /// whether we use clean urls or not. Some example output would be
    /// "", "../", "../../../", "../index.html", etc.
    pub fn upwards_href(&self, levels: usize) -> String {
        if levels > 0 {
            match self.clean_urls {
                true => "../".repeat(levels),
                false => "../".repeat(levels) + "index.html"
            }
        } else {
            String::from("")
        }
    }

    /// Writes `site.eno` in the site directory
    pub fn write_manifest(&self, context: &Context) {
        let mut eno = String::new();

        if let Some(base_url) = &self.base_url {
            eno.push_str(&format!("base_url: {}\n", base_url.representation));
        }

        if self.deploy_config.server.is_some() ||
            self.deploy_config.user.is_some() {
            eno.push_str("deploy_config:\n");

            if let Some(path) = &self.deploy_config.path {
                eno.push_str(&format!("path = {}\n", path));
            }

            if let Some(server) = &self.deploy_config.server {
                eno.push_str(&format!("server = {}\n", server));
            }

            if let Some(user) = &self.deploy_config.user {
                eno.push_str(&format!("user = {}\n", user));
            }
        }

        if !self.clean_urls {
            eno.push_str("disable_clean_urls\n");
        }

        if let Font::Custom(file_name) = &self.font {
            eno.push_str(&format!("font: {}\n", file_name));
        }

        if !self.language.default {
            eno.push_str(&format!("language: {}\n", self.language.code));
        }

        if let AspectRatio::Custom { representation, .. } = &self.poster_aspect {
            eno.push_str(&format!("poster_aspect: {}\n", representation));
        }

        eno.push_str(&format!("theme: {}\n", self.theme.key));

        let path = context.site_dir.join("site.eno");
        fs::write(path, eno).unwrap();
    }
}

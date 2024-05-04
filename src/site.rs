use std::fs;
use std::path::Path;
use std::sync::Arc;

use enolib::{Element, SectionElement};
use url::Url;

use crate::{
    Context,
    Collection,
    Font,
    GetAnyResult,
    HyperDir,
    Language,
    Playlist,
    Theme,
    Video
};
use crate::util::supported_font_extension;

#[derive(Clone, Debug)]
pub enum AspectRatio {
    Custom {
        representation: String,
        value: f32
    },
    Default
}

/// Base url can be user-provided with or without trailing slash
/// ("https://example.com/subdir" "https://example.com/subdir/"), but for
/// further url construction through the url crate the trailing slash is
/// significant (https://docs.rs/url/latest/url/struct.Url.html#method.join),
/// hence we store both representation and a working copy of the parsed url.
#[derive(Clone, Debug)]
pub struct BaseUrl {
    pub representation: String,
    pub value: Url
}

#[derive(Clone, Debug)]
pub struct DeployConfig {
    pub path: Option<String>,
    pub server: Option<String>,
    pub user: Option<String>
}

#[derive(Clone, Debug)]
pub struct Site {
    pub base_url: Option<BaseUrl>,
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

impl BaseUrl {
    /// Delegates parsing to the url crate but beforehand ensures a trailing
    /// slash so further url construction is performed correctly.
    pub fn parse(url: &str) -> Result<BaseUrl, String> {
        let parsed_url = match url.ends_with('/') {
            true => Url::parse(url),
            false => Url::parse(&format!("{url}/"))
        };

        match parsed_url {
            Ok(value) => {
                Ok(BaseUrl {
                    representation: url.to_owned(),
                    value
                })
            }
            Err(err) => Err(err.to_string())
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

        match document.optional_field("base_url") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(value) => match BaseUrl::parse(&value) {
                        Ok(url) => self.base_url = Some(url),
                        Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), field.line_number(), err))
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_flag("disable_clean_urls") {
            Ok(Some(_)) => {
                self.clean_urls = false;
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("deploy_config") {
            Ok(Some(field)) => match field.attributes() {
                Ok(attributes) => {
                    for attribute in attributes {
                        match attribute.key() {
                            "path" => match attribute.required_value::<String>() {
                                Ok(path) => self.deploy_config.path = Some(path),
                                Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err.message))
                            }
                            "server" => match attribute.required_value::<String>() {
                                Ok(server) => self.deploy_config.server = Some(server),
                                Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err.message))
                            }
                            "user" => match attribute.required_value::<String>() {
                                Ok(user) => self.deploy_config.user = Some(user),
                                Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err.message))
                            }
                            _ => self.errors.push(format!("Ignoring unsupported attribute '{}' in {}:{}", attribute.key(), manifest_path.display(), attribute.line_number()))
                        }
                    }
                }
                Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), field.line_number()))
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("font") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(file_name) => {
                        let path_absolute = manifest_path.parent().unwrap().join(&file_name);
                        if path_absolute.exists() {
                            if supported_font_extension(&file_name) {
                                self.font = Font::Custom(file_name);
                            } else {
                                self.errors.push(format!("Error in {}:{} (Font file extension is not supported, only woff and woff2 are)", manifest_path.display(), field.line_number()));
                            }
                        } else {
                            self.errors.push(format!("Error in {}:{} (Path to font is invalid, file not found)", manifest_path.display(), field.line_number()));
                        }
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("language") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(value) => {
                        self.language = Language::from_code(&value);
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("poster_aspect") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(representation) => {
                        match AspectRatio::from_representation(&representation) {
                            Ok(aspect_ratio) => self.poster_aspect = aspect_ratio,
                            Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), field.line_number()))
                        }
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("theme") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(value) => {
                        match Theme::from_key(value.as_str()) {
                            Ok(theme) => {
                                self.theme = theme;
                            }
                            Err(err) => {
                                self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), field.line_number(), err));
                            }
                        }
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        for element in &document.untouched_elements() {
            if let Some(attribute) = element.as_attribute() {
                self.errors.push(format!("Ignoring unsupported attribute '{}' in {}:{}", attribute.key(), manifest_path.display(), element.line_number()))
            } else if let Some(embed) = element.as_embed() {
                self.errors.push(format!("Ignoring unsupported embed '{}' in {}:{}", embed.key(), manifest_path.display(), element.line_number()))
            } else if let Some(flag) = element.as_flag() {
                self.errors.push(format!("Ignoring unsupported flag '{}' in {}:{}", flag.key(), manifest_path.display(), element.line_number()))
            } else if let Some(field) = element.as_field() {
                self.errors.push(format!("Ignoring unsupported field '{}' in {}:{}", field.key(), manifest_path.display(), element.line_number()))
            } else if let Some(section) = element.as_section() {
                self.errors.push(format!("Ignoring unsupported section '{}' in {}:{}", section.key(), manifest_path.display(), element.line_number()))
            }
        }
    }

    /// Returns a collection, playlist or video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        if path == "" {
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
            if path == "" {
                return Some(Arc::clone(collection));
            } else {
                return collection.get_collection(path);
            }
        }

        None
    }

    /// Returns the playlist at the given path.
    pub fn get_playlist(&self, path: &str) -> Option<Arc<Playlist>> {
        if path == "" {
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
        if path == "" {
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

        if hyper_dir.collection_manifest.is_some() {
            if hyper_dir.playlist_manifest.is_some() || hyper_dir.video_manifest.is_some() {
                let error = "Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.".to_string();
                site.errors.push(error);
                site.content = SiteContent::Invalid;
            } else {
                let collection = Collection::read_dir(context, &hyper_dir, &site);
                site.content = SiteContent::Collection(collection);
            }
        } else if hyper_dir.playlist_manifest.is_some() {
            if hyper_dir.video_manifest.is_some() {
                let error = "Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.".to_string();
                site.errors.push(error);
                site.content = SiteContent::Invalid;
            } else {
                let playlist = Playlist::read_dir(context, &hyper_dir, &site);
                site.content = SiteContent::Playlist(playlist);
            }
        } else if hyper_dir.video_manifest.is_some() {
            let video = Video::read_dir(context, &hyper_dir, &site);
            site.content = SiteContent::Video(video);
        } else if !hyper_dir.dirs.is_empty() {
            eprintln!("No manifest in the site directory. Going forward, a collection is assumed (because there are folders inside).");
            let collection = Collection::read_dir(context, &hyper_dir, &site);
            site.content = SiteContent::Collection(collection);
        } else if !hyper_dir.video_files.is_empty() {
            eprintln!("No manifest in the site directory. Going forward, a video is assumed (because there is a/are video file(s) inside).");
            let video = Video::read_dir(context, &hyper_dir, &site);
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

    /// Writes `site.eno` in the site directory
    pub fn write_manifest(&self, site_dir: &Path) {
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

        let path = site_dir.join("site.eno");
        fs::write(path, eno).unwrap();
    }
}

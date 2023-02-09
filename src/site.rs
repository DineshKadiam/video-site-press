use std::fs;
use std::path::Path;
use std::sync::Arc;

use enolib::SectionElement;

use crate::{
    Context,
    Collection,
    GetAnyResult,
    HyperDir,
    Playlist,
    Theme,
    Video
};

#[derive(Clone, Debug)]
pub struct Site {
    pub base_url: Option<String>,
    pub content: SiteContent,
    pub errors: Vec<String>,
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
                match field.required_value() {
                    Ok(value) => {
                        self.base_url = Some(value);
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
                        match value.as_str() {
                            "dark" => {
                                self.theme = Theme::dark();
                            }
                            "light" => {
                                self.theme = Theme::light();
                            }
                            _ => self.errors.push(format!("Ignoring unknown theme option '{}' in {}", value, manifest_path.display()))
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

    pub fn new() -> Site {
        Site {
            base_url: None,
            content: SiteContent::Empty,
            errors: Vec::new(),
            theme: Theme::default()
        }
    }

    pub fn persist_in_site(&self, site_dir: &Path) {
        let mut eno = String::new();

        if let Some(base_url) = &self.base_url {
            eno.push_str(&format!("base_url: {}\n", base_url));
        }

        eno.push_str(&format!("theme: {}\n", self.theme.key));

        let path = site_dir.join("site.eno");
        fs::write(path, eno).unwrap();
    }

    pub fn read_dir(context: &Context, dir: &Path) -> Arc<Site> {
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
                let collection = Collection::read_dir(context, &hyper_dir);
                site.content = SiteContent::Collection(collection);
            }
        } else if hyper_dir.playlist_manifest.is_some() {
            if hyper_dir.video_manifest.is_some() {
                let error = "Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.".to_string();
                site.errors.push(error);
                site.content = SiteContent::Invalid;
            } else {
                let playlist = Playlist::read_dir(context, &hyper_dir);
                site.content = SiteContent::Playlist(playlist);
            }
        } else if hyper_dir.video_manifest.is_some() {
            let video = Video::read_dir(context, &hyper_dir);
            site.content = SiteContent::Video(video);
        } else if !hyper_dir.dirs.is_empty() {
            eprintln!("No manifest in the site directory. Going forward, a collection is assumed (because there are folders inside).");
            let collection = Collection::read_dir(context, &hyper_dir);
            site.content = SiteContent::Collection(collection);
        } else if !hyper_dir.video_files.is_empty() {
            eprintln!("No manifest in the site directory. Going forward, a video is assumed (because there is a/are video file(s) inside).");
            let video = Video::read_dir(context, &hyper_dir);
            site.content = SiteContent::Video(video);
        }

        Arc::new(site)
    }

    pub fn title(&self) -> &str {
        match &self.content {
            SiteContent::Collection(collection) => collection.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Playlist(playlist) => playlist.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Video(video) => video.title.as_deref().unwrap_or("Hyper 8"),
            _ => "Hyper 8"
        }
    }
}

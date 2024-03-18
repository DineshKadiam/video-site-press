use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use enolib::SectionElement;
use serde_derive::{Deserialize, Serialize};

use crate::{
    BrowserSupport,
    CacheItem,
    Collection,
    Context,
    FileMeta,
    HyperDir,
    Playlist,
    PosterFile,
    VideoMetaCached
};
use crate::util::hash;

#[derive(Clone, Debug)]
pub struct Video {
    pub description: Option<String>,
    pub errors: Vec<String>,
    /// Relative to site dir, always without leading/trailing slashes
    pub path: String,
    pub poster: Option<PosterFile>,
    pub release_date: Option<NaiveDate>,
    pub title: Option<String>,
    /// Unlisted videos are not linked from the site anywhere,
    /// one needs to know the link to access them.
    pub unlisted: bool,
    pub versions: Vec<VideoFile>
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct VideoAssets {
    // pub requested: Vec<VideoJob>
// }

// pub enum <T>Foo {
//     Unavailable,
//     Processing { process_id: usize },
//     Available(T)
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct VideoJob {
//     pub assigned: bool,
//     pub format: VideoFormat,
//     pub success: Option<Result<(), String>>
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct VideoJobType {
//     pub assigned: bool,
//     pub format: VideoFormat,
//     pub success: Option<Result<(), String>>
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoFile {
    pub file_meta: FileMeta,
    pub file_name: String,
    pub video_meta: Option<VideoMeta>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoMeta {
    pub browser_support: BrowserSupport,
    /// Duration in seconds
    pub duration: f32,
    pub has_audio: bool,
    pub height: usize,
    pub width: usize
}

pub enum VideoParent {
    Collection(Arc<Collection>),
    Playlist(Arc<Playlist>)
}

impl Video {
    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return
            }
        };

        match document.optional_embed("description") {
            Ok(Some(field)) => {
                match field.required_value() {
                    Ok(value) => {
                        self.description = Some(value);
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("poster") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(file_name) => {
                        let path_absolute = manifest_path.parent().unwrap().join(&file_name);
                        if path_absolute.exists() {
                            let file_meta = FileMeta::new(&path_absolute);
                            let assets = context.cache.lock().unwrap().fetch_poster_assets(&file_meta, &file_name, &self.path);
                            let poster_file = PosterFile::new(assets, file_meta, file_name);

                            self.poster = Some(poster_file);
                        } else {
                            self.errors.push(format!("Error in {} (Path to poster is invalid, file not found)", manifest_path.display()));
                        }
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("release_date") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(value) => match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                        Ok(date) => {
                            self.release_date = Some(date);
                        }
                        Err(err) => self.errors.push(format!("Error in {} ({})", manifest_path.display(), err))
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_field("title") {
            Ok(Some(field)) => {
                match field.required_value() {
                    Ok(value) => {
                        self.title = Some(value);
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err))
                }
            }
            Err(err) => self.errors.push(format!("Error in {}:{} ({})", manifest_path.display(), err.line, err)),
            _ => ()
        }

        match document.optional_flag("unlisted") {
            Ok(Some(_)) => {
                self.unlisted = true;
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

    // pub fn has_format(&self, format: &VideoFormat) -> bool {
    //     self.assets.versions
    //         .iter()
    //         .any(|version| &version.format == format)
    // }

    pub fn highest_resolution_version(&self) -> Option<&VideoFile> {
        let mut version_candidate: Option<(usize, &VideoFile)> = None;

        for version in &self.versions {
            if let Some(video_meta) = &version.video_meta {
                let pixels = video_meta.width * video_meta.height;
                if version_candidate.is_none() || version_candidate.is_some_and(|pixels_version| pixels > pixels_version.0) {
                    version_candidate = Some((pixels, version));
                }
            }
        }

        version_candidate.map(|pixels_version| pixels_version.1)
    }

    pub fn label(&self) -> &str {
        self.title
            .as_deref()
            .unwrap_or_else(||
                self.slug_opt()
                    .unwrap_or("Video")
            )
    }

    fn new(path: String) -> Video {
        Video {
            description: None,
            errors: Vec::new(),
            path,
            poster: None,
            release_date: None,
            title: None,
            unlisted: false,
            versions: Vec::new()
        }
    }

    pub fn persist_in_cache(&self, cache_dir: &Path) {
        for version in &self.versions {
            if let Some(video_meta) = &version.video_meta {
                let cache_item = CacheItem::VideoMeta(VideoMetaCached {
                    file_meta: version.file_meta.clone(),
                    path: format!("{}/{}", self.path, version.file_name),
                    video_meta: video_meta.clone()
                });

                let serialized = bincode::serialize(&cache_item).unwrap();

                let version_path = format!("{}/{}", self.path, version.file_name);
                let file_name = format!("{}.bincode", hash(&version_path));
                let path = cache_dir.join(file_name);
                
                fs::write(path, serialized).unwrap();
            }
        }
    }

    pub fn read_dir(context: &Context, hyper_dir: &HyperDir) -> Arc<Video> {
        let mut video = Video::new(hyper_dir.site_path(context));

        if let Some(manifest) = &hyper_dir.video_manifest {
            video.apply_manifest(context, manifest);
        }

        if !hyper_dir.dirs.is_empty() {
            let folder_names = hyper_dir.dirs
                .iter()
                .map(|dir| format!("'{}'", dir.file_name().unwrap().to_string_lossy()))
                .collect::<Vec<String>>()
                .join(", ");

            video.errors.push(format!("Ignoring folders {folder_names} inside video folder {} (only files may be placed inside a video folder).", hyper_dir.path.display()));
        }

        // TODO: Some strategy what we do with multiple images (e.g. heuristic to pick most probable poster, use additional ones, etc.)
        if !hyper_dir.image_files.is_empty() && video.poster.is_none() {
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let assets = context.cache.lock().unwrap().fetch_poster_assets(&file_meta, &file_name, &video.path);

            let poster_file = PosterFile::new(assets, file_meta, file_name);

            video.poster = Some(poster_file);
        }

        for video_file in &hyper_dir.video_files {
            let file_meta = FileMeta::new(video_file);
            let file_name = video_file.file_name().unwrap().to_string_lossy().to_string();

            let version_path = format!("{}/{}", &video.path, file_name);
            let video_meta = context.cache.lock().unwrap().video_meta
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
                .map(|cached| cached.video_meta.clone());

            video.versions.push(VideoFile::new(file_meta, file_name, video_meta));
        }

        if !hyper_dir.errors.is_empty() {
            let mut error = "Directories and/or files inside this video directory have issues:".to_string();

            for message in &hyper_dir.errors {
                error.push_str(&format!("\n- {}", message));
            }

            video.errors.push(error);
        }

        Arc::new(video)
    }

    pub fn slug(&self) -> &str {
        self.path.split('/').last().unwrap()
    }

    pub fn slug_opt(&self) -> Option<&str> {
        if self.path == "" {
            None
        } else {
            self.path.split('/').last()
        }
    }

    // TODO: It could happen that we delete a file while it is being transcoded.
    //       How do we handle/prevent this?
    pub fn unpersist(&self, context: &Context) {
        if self.path == "" {
            let manifest_path = context.site_dir.join("video.eno");
            if manifest_path.exists() {
                fs::remove_file(&manifest_path).unwrap();
            }

            if let Some(poster) = &self.poster {
                let poster_path = context.site_dir.join(&poster.file_name);
                if poster_path.exists() {
                    fs::remove_file(&poster_path).unwrap();
                }
            }

            for version in &self.versions {
                let version_path = context.site_dir.join(&version.file_name);
                if version_path.exists() {
                    fs::remove_file(&version_path).unwrap();
                }
            }
        } else {
            // Guard against disastrous recursive folder removal from concatenating absolute paths
            assert!(!self.path.starts_with('/'), "Recursive folder removal aborted due to dangerous circumstances");
            let video_dir = context.site_dir.join(&self.path);
            fs::remove_dir_all(video_dir).unwrap();
        }
    }

    pub fn write_manifest(&self, context: &Context) {
        let mut eno = String::new();

        if let Some(poster) = &self.poster {
            eno.push_str(&format!("poster: {}\n", poster.file_name));
        }

        if let Some(release_date) = &self.release_date {
            eno.push_str(&format!("release_date: {}\n", release_date));
        }

        if let Some(title) = &self.title {
            eno.push_str(&format!("title: {}\n", title));
        }

        if self.unlisted {
            eno.push_str("unlisted\n");
        }

        if let Some(description) = &self.description {
            if !eno.is_empty() { eno.push('\n'); }
            eno.push_str(&format!("-- description\n{}\n--description\n", description));
        }

        let path = context.site_dir.join(&self.path).join("video.eno");
        fs::write(path, eno).unwrap();
    }
}

impl VideoFile {
    pub fn new(
        file_meta: FileMeta,
        file_name: String,
        video_meta: Option<VideoMeta>
    ) -> VideoFile {
        VideoFile {
            file_meta,
            file_name,
            video_meta
        }
    }
}

impl VideoMeta {
    pub fn new(
        browser_support: BrowserSupport,
        duration: f32,
        has_audio: bool,
        height: usize,
        width: usize
    ) -> VideoMeta {
        VideoMeta {
            browser_support,
            duration,
            has_audio,
            height,
            width
        }
    }
}

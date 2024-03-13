use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::str::Split;
use std::sync::Arc;

use enolib::SectionElement;

use crate::{
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    PosterFile,
    Video
};

#[derive(Clone, Debug)]
pub struct Playlist {
	pub description: Option<String>,
    pub errors: Vec<String>,
    pub path: String,
	pub poster: Option<PosterFile>,
	pub title: Option<String>,
    /// Unlisted playlists are not linked from the site anywhere,
    /// one needs to know the link to access them.
    pub unlisted: bool,
	pub videos_physical: Vec<Arc<Video>>,
    pub videos_virtual: Vec<String>
}

impl Playlist {
    /// TODO:
    /// - poster support (image description/path)
    /// - video listing
    fn apply_manifest(&mut self, manifest_path: &Path) {
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
                match field.required_value() {
                    Ok(value) => {
                        let path_absolute = manifest_path.parent().unwrap().join(&value);
                        if path_absolute.exists() {
                            let file_meta = FileMeta::new(&path_absolute);
                            let poster_file = PosterFile::new(None, file_meta, value);
                            self.poster = Some(poster_file);
                        } else {
                            // TODO: Error
                        }
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

    pub fn delete_video(
        &mut self,
        context: &Context,
        mut path_parts: Peekable<Split<char>>
    ) -> bool {
        if let Some(slug) = path_parts.next() {
            if let Some(video) = self.videos_physical
                .iter()
                .position(|video| video.slug() == slug)
                .map(|index| self.videos_physical.swap_remove(index)) {
                video.unpersist(context);
                return true;
            }
        }

        false
    }

    /// Potentially returns a video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        for video in &self.videos_physical {
            if video.path == path {
                return GetAnyResult::Video(Arc::clone(video));
            }
        }

        GetAnyResult::None
    }

    /// Returns the video at the given path.
    pub fn get_video(&self, path: &str) -> Option<Arc<Video>> {
        for video in &self.videos_physical {
            if video.path == path {
                return Some(Arc::clone(video));
            }
        }

        None
    }

    pub fn label(&self) -> &str {
        self.title
            .as_deref()
            .unwrap_or_else(||
                self.slug_opt()
                    .unwrap_or("Playlist")
            )
    }

	pub fn new(path: String) -> Playlist {
		Playlist {
            description: None,
            errors: Vec::new(),
            path,
            poster: None,
            title: None,
            videos_physical: Vec::new(),
            videos_virtual: Vec::new(),
            unlisted: false
        }
	}

    pub fn persist_in_site(&self, site_dir: &Path) {
        let mut eno = String::new();

        if let Some(poster) = &self.poster {
            eno.push_str(&format!("poster: {}\n", poster.file_name));
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

        let path = site_dir.join(&self.path).join("playlist.eno");
        fs::write(path, eno).unwrap();
    }

    pub fn read_dir(context: &Context, hyper_dir: &HyperDir) -> Arc<Playlist> {
        let mut playlist = Playlist::new(hyper_dir.site_path(context));

        if let Some(manifest) = &hyper_dir.playlist_manifest {
            playlist.apply_manifest(manifest);
        }

        for subdir in &hyper_dir.dirs {
            let hyper_subdir = HyperDir::read(subdir);
            playlist.read_subdir(context, &hyper_subdir);
        }

        if !hyper_dir.image_files.is_empty() {
            // TODO: Some strategy what we do with multiple images (e.g. heuristic to pick most probable poster, use additional ones, etc.)
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(&first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let poster_path = format!("{}/{}", &playlist.path, file_name);
            let assets = context.cache.lock().unwrap().poster_assets
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == poster_path)
                .map(|cached| cached.assets.clone());

            let poster_file = PosterFile::new(assets, file_meta, file_name);

            playlist.poster = Some(poster_file);
        }

        if !hyper_dir.video_files.is_empty() {
            let mut error = "No video files should be directly in a playlist directory, maybe you wanted to put them in subfolder instead?".to_string();

            for video_file in &hyper_dir.video_files {
                error.push_str(&format!("\n- {}", video_file.display()));
            }

            playlist.errors.push(error);
        }

        Arc::new(playlist)
    }

    fn read_subdir(&mut self, context: &Context, hyper_subdir: &HyperDir) {
        if hyper_subdir.collection_manifest.is_some() {
            let error = format!("Ignoring playlist subdirectory {} because it contains collection.eno (only video.eno may exist in a playlist subdirectory).", hyper_subdir.path.display());
            self.errors.push(error);
        } else if hyper_subdir.playlist_manifest.is_some() {
            let error = format!("Ignoring playlist subdirectory {} because it contains playlist.eno (only video.eno may exist in a playlist subdirectory).", hyper_subdir.path.display());
            self.errors.push(error);
        } else if hyper_subdir.video_manifest.is_some() ||
            !hyper_subdir.video_files.is_empty() ||
            !hyper_subdir.audio_files.is_empty() {
            let video = Video::read_dir(context, hyper_subdir);
            self.videos_physical.push(video);
        }
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

    pub fn unpersist_in_site(&self, context: &Context) {
        if self.path == "" {
            let manifest_path = context.site_dir.join("playlist.eno");
            if manifest_path.exists() {
                fs::remove_file(&manifest_path).unwrap();
            }

            if let Some(poster) = &self.poster {
                let poster_path = context.site_dir.join(&poster.file_name);
                if poster_path.exists() {
                    fs::remove_file(&poster_path).unwrap();
                }
            }

            for video in &self.videos_physical {
                let video_path = context.site_dir.join(video.slug());
                if video_path.exists() {
                    fs::remove_dir_all(&video_path).unwrap();
                }
            }
        } else {
            // Guard against disastrous recursive folder removal from concatenating absolute paths
            assert!(!self.path.starts_with('/'), "Recursive folder removal aborted due to dangerous circumstances");
            let playlist_dir = context.site_dir.join(&self.path);
            fs::remove_dir_all(playlist_dir).unwrap();
        }
    }

    /// Intended to be called with at least one remaining path part to consume,
    /// otherwise will panic.
    pub fn update_from_filesystem(&mut self, context: &Context, mut path_parts: Peekable<Split<char>>) {
        let slug = path_parts.next().unwrap();

        if path_parts.peek().is_some() {
            // A video (and only video folders may exist in a playlist) isn't allowed
            // to have subfolders, so this is a bit of an odd case.

            // Remove a video at this path if there is one
            if let Some(index) = self.videos_physical.iter().position(|video| video.slug() == slug) {
                self.videos_physical.swap_remove(index);
            }

            let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
            self.read_subdir(context, &hyper_subdir);
        } else {
            // If we presently have a video at the given path, we remove it.
            if let Some(index) = self.videos_physical.iter().position(|video| video.slug() == slug) {
                self.videos_physical.swap_remove(index);
            }

            // Add the new leaf (respectively, replace what was removed)
            let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
            self.read_subdir(context, &hyper_subdir);
        }
    }

    pub fn update_video(
        &mut self,
        mutation: impl FnOnce(&mut Video),
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Arc<Video>, ()> {
        if let Some(slug) = path_parts.next() {
            if let Some(video) = self.videos_physical.iter_mut().find(|video| video.slug() == slug) {
                let mut video_mut = (**video).clone();
                mutation(&mut video_mut);
                *video = Arc::new(video_mut);
                return Ok(Arc::clone(video));
            }
        }

        Err(())
    }
}
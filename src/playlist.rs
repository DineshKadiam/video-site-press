use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::str::Split;
use std::sync::Arc;

use enolib::{Element, SectionElement};

use crate::{
    Banner,
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    Site,
    Video
};

#[derive(Clone, Debug)]
pub struct Playlist {
	pub banner: Option<Banner>,
	pub description: Option<String>,
    pub errors: Vec<String>,
    pub path: String,
	pub title: Option<String>,
    /// Unlisted playlists are not linked from the site anywhere,
    /// one needs to know the link to access them.
    pub unlisted: bool,
	pub videos: Vec<Arc<Video>>,
    // TODO: We could potentially drop this and follow up with the plan of
    //       making video order simply a sorting matter, i.e. based on video release_date
    //       and/or a user-assigned video "number" respectively.
    pub videos_virtual: Vec<String>
}

impl Playlist {
    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        // TODO: Tricky manifest problem/question in general: If there is anything wrong
        //       with the manifest (especially: syntax error rendering the entire manifest useless),
        //       we can potentially erase everything contained in it by doing an update via the browser
        //       editor (it persists what has been read - nothing!). Consequently we should e.g. disable
        //       the edit form/updates via browser editor while there are issues with the manifest, or
        //       try to read in some things in a "broken state", e.g. registering a poster although the
        //       image referenced was not found.

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

        match document.optional_field("banner") {
            Ok(Some(field)) => {
                match field.required_attribute("file") {
                    Ok(attribute) => {
                        match attribute.required_value::<String>() {
                            Ok(file_name) => {
                                let path_absolute = manifest_path.parent().unwrap().join(&file_name);
                                if path_absolute.exists() {
                                    let file_meta = FileMeta::new(&path_absolute);

                                    // TODO: Errors
                                    let description = match field.required_attribute("description") {
                                        Ok(attribute) => match attribute.required_value() {
                                            Ok(description) => Some(description),
                                            _ => None
                                        }
                                        _ => None
                                    };

                                    let assets = context.cache.lock().unwrap().fetch_poster_assets(
                                        &file_meta,
                                        &file_name,
                                        &self.path,
                                        0.0 // TODO: Banners have no poster_aspect, pending better modeling we just hard-code it to 0 always
                                    );

                                    let banner = Banner::new(assets, description, file_meta, file_name);
                                    self.banner = Some(banner);
                                } else {
                                    self.errors.push(format!("Error in {}:{} (Path to banner is invalid, file not found)", manifest_path.display(), attribute.line_number()));
                                }
                            }
                            Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), field.line_number()))
                        }
                    }
                    Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), field.line_number()))
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
            if let Some(video) = self.videos
                .iter()
                .position(|video| video.slug() == slug)
                .map(|index| self.videos.swap_remove(index)) {
                video.unpersist(context);
                return true;
            }
        }

        false
    }

    /// Potentially returns a video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        for video in &self.videos {
            if video.path == path {
                return GetAnyResult::Video(Arc::clone(video));
            }
        }

        GetAnyResult::None
    }

    /// Returns the video at the given path.
    pub fn get_video(&self, path: &str) -> Option<Arc<Video>> {
        for video in &self.videos {
            if video.path == path {
                return Some(Arc::clone(video));
            }
        }

        None
    }

    pub fn label<'a>(&'a self, context: &'a Context) -> &str {
        self.title
            .as_deref()
            .unwrap_or_else(||
                self.slug_opt()
                    .unwrap_or(&context.editor_translations.playlist)
            )
    }

	pub fn new(path: String) -> Playlist {
		Playlist {
            banner: None,
            description: None,
            errors: Vec::new(),
            path,
            title: None,
            videos: Vec::new(),
            videos_virtual: Vec::new(),
            unlisted: false
        }
	}

    pub fn read_dir(context: &Context, hyper_dir: &HyperDir, site: &Site) -> Arc<Playlist> {
        let mut playlist = Playlist::new(hyper_dir.site_path(context));

        if let Some(manifest) = &hyper_dir.playlist_manifest {
            playlist.apply_manifest(context, manifest);
        }

        for subdir in &hyper_dir.dirs {
            let hyper_subdir = HyperDir::read(subdir);
            playlist.read_subdir(context, &hyper_subdir, site);
        }

        // TODO: Some strategy what we do with multiple images (e.g. heuristic to pick most probable banner, use additional ones, etc.)
        if !hyper_dir.image_files.is_empty() && playlist.banner.is_none() {
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let assets = context.cache.lock().unwrap().fetch_poster_assets(
                &file_meta,
                &file_name,
                &playlist.path,
                0.0
            );

            let banner = Banner::new(assets, None, file_meta, file_name);

            playlist.banner = Some(banner);
        }

        if !hyper_dir.video_files.is_empty() {
            let mut error = "No video files should be directly in a playlist directory, maybe you wanted to put them in subfolder instead?".to_string();

            for video_file in &hyper_dir.video_files {
                error.push_str(&format!("\n- {}", video_file.display()));
            }

            playlist.errors.push(error);
        }

        if !hyper_dir.errors.is_empty() {
            let mut error = "Directories and/or files inside this playlist directory have issues:".to_string();

            for message in &hyper_dir.errors {
                error.push_str(&format!("\n- {}", message));
            }

            playlist.errors.push(error);
        }

        Arc::new(playlist)
    }

    fn read_subdir(&mut self, context: &Context, hyper_subdir: &HyperDir, site: &Site) {
        if hyper_subdir.collection_manifest.is_some() {
            let error = format!("Ignoring playlist subdirectory {} because it contains collection.eno (only video.eno may exist in a playlist subdirectory).", hyper_subdir.path.display());
            self.errors.push(error);
        } else if hyper_subdir.playlist_manifest.is_some() {
            let error = format!("Ignoring playlist subdirectory {} because it contains playlist.eno (only video.eno may exist in a playlist subdirectory).", hyper_subdir.path.display());
            self.errors.push(error);
        } else if hyper_subdir.video_manifest.is_some() ||
            !hyper_subdir.video_files.is_empty() ||
            !hyper_subdir.audio_files.is_empty() {
            let video = Video::read_dir(context, hyper_subdir, site);
            self.videos.push(video);
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

    pub fn unpersist(&self, context: &Context) {
        if self.path == "" {
            let manifest_path = context.site_dir.join("playlist.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(banner) = &self.banner {
                let banner_path = context.site_dir.join(&banner.file_name);
                let _ = fs::remove_file(&banner_path);
            }

            for video in &self.videos {
                let video_path = context.site_dir.join(video.slug());
                let _ = fs::remove_dir_all(&video_path);
            }
        } else {
            // Guard against disastrous recursive folder removal from concatenating absolute paths
            assert!(!self.path.starts_with('/'), "Recursive folder removal aborted due to dangerous circumstances");
            let playlist_dir = context.site_dir.join(&self.path);
            let _ = fs::remove_dir_all(playlist_dir);
        }
    }

    /// Intended to be called with at least one remaining path part to consume,
    /// otherwise will panic.
    pub fn update_from_filesystem(
        &mut self,
        context: &Context,
        mut path_parts: Peekable<Split<char>>,
        site: &Site
    ) {
        let slug = path_parts.next().unwrap();

        if path_parts.peek().is_some() {
            // A video (and only video folders may exist in a playlist) isn't allowed
            // to have subfolders, so this is a bit of an odd case.

            // Remove a video at this path if there is one
            if let Some(index) = self.videos.iter().position(|video| video.slug() == slug) {
                self.videos.swap_remove(index);
            }

            let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
            self.read_subdir(context, &hyper_subdir, site);
        } else {
            // If we presently have a video at the given path, we remove it.
            if let Some(index) = self.videos.iter().position(|video| video.slug() == slug) {
                self.videos.swap_remove(index);
            }

            // Add the new leaf (respectively, replace what was removed)
            let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
            self.read_subdir(context, &hyper_subdir, site);
        }
    }

    /// Performs the passed mutation on the video under the given path (if it exists).
    /// Should not be called directly, instead only through a cascade call originating
    /// from `update_video` on [Context].
    pub fn update_video(
        &mut self,
        mutation: impl FnOnce(&mut Video),
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Arc<Video>, ()> {
        if let Some(slug) = path_parts.next() {
            if let Some(video) = self.videos.iter_mut().find(|video| video.slug() == slug) {
                let mut video_mut = (**video).clone();
                mutation(&mut video_mut);
                *video = Arc::new(video_mut);
                return Ok(Arc::clone(video));
            }
        }

        Err(())
    }

    pub fn write_manifest(&self, site_dir: &Path) {
        let mut eno = String::new();

        if let Some(banner) = &self.banner {
            eno.push_str("banner:\n");

            if let Some(description) = &banner.description {
                eno.push_str(&format!("description = {}\n", description));
            }

            eno.push_str(&format!("file = {}\n", banner.file_name));
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
}
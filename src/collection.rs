use std::cmp;
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::str::Split;
use std::sync::Arc;

use chrono::NaiveDate;
use enolib::{Element, SectionElement};

use crate::{
    Banner,
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    HyperUrl,
    Link,
    Playlist,
    Site,
    Translations,
    Video,
    VideoParent
};
use crate::util::checked_remove_dir_all;

#[derive(Clone, Debug)]
pub struct Collection {
    pub banner: Option<Banner>,
    pub description: Option<String>,
    pub errors: Vec<String>,
    pub links: Vec<Link>,
    pub path: String,
    pub playlists: Vec<Arc<Playlist>>,
    pub subcollections: Vec<Arc<Collection>>,
    pub title: Option<String>,
    pub unlisted: bool,
    pub videos: Vec<Arc<Video>>
}

impl Collection {
    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return
            }
        };

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

        // TODO: Encapsulate in link.rs maybe?
        // TODO: Would benefit from a .fields() iterator (then again errors for inaccurately typed errors would be harder to give)
        for element in document.elements() {
            if element.key() == "link" {
                if let Some(field) = element.as_field() {
                    match field.required_attribute("url") {
                        Ok(attribute) => {
                            match attribute.required_value::<String>() {
                                Ok(value) => match HyperUrl::parse(&value) {
                                    Ok(url) => {
                                        // TODO: Errors, optional_attribute does not exist in enolib?
                                        let label = match field.required_attribute("label") {
                                            Ok(attribute) => match attribute.required_value() {
                                                Ok(label) => Some(label),
                                                Err(_) => None
                                            }
                                            Err(_) => None
                                        };

                                        let (hidden, rel_me) = match field.required_attribute("verification") {
                                            Ok(attribute) => match attribute.required_value::<String>() {
                                                Ok(value) => match value.as_str() {
                                                    "rel-me" => (false, true),
                                                    "rel-me-hidden" => (true, true),
                                                    _ => (false, false)
                                                }
                                                Err(_) => (false, false)
                                            }
                                            Err(_) => (false, false)
                                        };

                                        let link = Link::new(hidden, label, rel_me, url);
                                        self.links.push(link);
                                    }
                                    Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), attribute.line_number()))
                                }
                                Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), err.line))
                            }
                        }
                        Err(err) => self.errors.push(format!("Error in {}:{} ({err})", manifest_path.display(), err.line))
                    }
                } else {
                    self.errors.push(format!("Error in {}:{} (Links must be provided as fields with attributes)", manifest_path.display(), element.line_number()))
                }
            }
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

    pub fn delete_collection(
        &mut self,
        context: &Context,
        mut path_parts: Peekable<Split<char>>
    ) -> bool {
        if let Some(slug) = path_parts.next() {
            if path_parts.peek().is_none() {
                if let Some(subcollection) = self.subcollections
                    .iter()
                    .position(|subcollection| subcollection.slug() == slug)
                    .map(|index| self.subcollections.swap_remove(index)) {
                    subcollection.unpersist(context);
                    return true;
                }
            } else if let Some(subcollection) = self.subcollections.iter_mut().find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();

                if subcollection_mut.delete_playlist(context, path_parts) {
                    *subcollection = Arc::new(subcollection_mut);
                    return true;
                }
            }
        }

        false
    }

    pub fn delete_playlist(
        &mut self,
        context: &Context,
        mut path_parts: Peekable<Split<char>>
    ) -> bool {
        if let Some(slug) = path_parts.next() {
            if let Some(playlist) = self.playlists
                .iter()
                .position(|playlist| playlist.slug() == slug)
                .map(|index| self.playlists.swap_remove(index)) {
                playlist.unpersist(context);
                return true;
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();

                if subcollection_mut.delete_playlist(context, path_parts) {
                    *subcollection = Arc::new(subcollection_mut);
                    return true;
                }
            }
        }

        false
    }

    /// Deletes the video at path (if found) and in case of success returns
    /// its deeper parent (subcollection or playlist or None if it was a
    /// direct child of this collection).
    pub fn delete_video(
        &mut self,
        context: &Context,
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Option<VideoParent>, ()> {
        if let Some(slug) = path_parts.next() {
            if let Some(video) = self.videos
                .iter()
                .position(|video| video.slug() == slug)
                .map(|index| self.videos.swap_remove(index)) {
                video.unpersist(context);
                return Ok(None);
            } else if let Some(playlist) = self.playlists
                .iter_mut()
                .find(|playlist| playlist.slug() == slug) {
                let mut playlist_mut = (**playlist).clone();

                if playlist_mut.delete_video(context, path_parts) {
                    *playlist = Arc::new(playlist_mut);
                    return Ok(Some(VideoParent::Playlist(Arc::clone(playlist))));
                }
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();

                if let Ok(video_parent) = subcollection_mut.delete_video(context, path_parts) {
                    match video_parent {
                        Some(video_parent) => {
                            *subcollection = Arc::new(subcollection_mut);
                            return Ok(Some(video_parent));
                        }
                        None => {
                            let subcollection_new = Arc::new(subcollection_mut);
                            *subcollection = Arc::clone(&subcollection_new);
                            return Ok(Some(VideoParent::Collection(subcollection_new)))
                        }
                    }
                }
            }
        }

        Err(())
    }

    /// Returns a collection, playlist or video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        for subcollection in &self.subcollections {
            if subcollection.path == path {
                return GetAnyResult::Collection(Arc::clone(subcollection));
            } else if path
                .strip_prefix(&subcollection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return subcollection.get_any(path);
            }
        }

        for playlist in &self.playlists {
            if playlist.path == path {
                return GetAnyResult::Playlist(Arc::clone(playlist));
            } else if path
                .strip_prefix(&playlist.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return playlist.get_any(path);
            }
        }

        for video in &self.videos {
            if video.path == path {
                return GetAnyResult::Video(Arc::clone(video));
            }
        }

        GetAnyResult::None
    }

    /// Returns the collection at the given path.
    pub fn get_collection(&self, path: &str) -> Option<Arc<Collection>> {
        for subcollection in &self.subcollections {
            if subcollection.path == path {
                return Some(Arc::clone(subcollection));
            } else if path
                .strip_prefix(&subcollection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return subcollection.get_collection(path);
            }
        }

        None
    }

    /// Returns the playlist at the given path.
    pub fn get_playlist(&self, path: &str) -> Option<Arc<Playlist>> {
        for playlist in &self.playlists {
            if playlist.path == path {
                return Some(Arc::clone(playlist));
            }
        }

        for subcollection in &self.subcollections {
            if path
                .strip_prefix(&subcollection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return subcollection.get_playlist(path);
            }
        }

        None
    }

    /// Returns the video at the given path.
    pub fn get_video(&self, path: &str) -> Option<Arc<Video>> {
        for video in &self.videos {
            if video.path == path {
                return Some(Arc::clone(video));
            }
        }

        for playlist in &self.playlists {
            if path
                .strip_prefix(&playlist.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return playlist.get_video(path);
            }
        }

        for subcollection in &self.subcollections {
            if path
                .strip_prefix(&subcollection.path)
                .is_some_and(|rest| rest.starts_with('/')) {
                return subcollection.get_video(path);
            }
        }

        None
    }

    pub fn label(&self, translations: &Translations) -> &str {
        self.title
            .as_deref()
            .unwrap_or_else(||
                self.slug_opt()
                    .unwrap_or(translations.collection)
            )
    }

    pub fn latest_release_date(&self) -> Option<NaiveDate> {
        let latest_in_playlists = self.public_playlists()
            .fold(None, |latest_release_date, playlist| {
                if let Some(release_date) = playlist.latest_release_date() {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::min(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        let latest_in_subcollections = self.public_subcollections()
            .fold(None, |latest_release_date, subcollection| {
                if let Some(release_date) = subcollection.latest_release_date() {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::min(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        let latest_in_videos = self.public_videos()
            .fold(None, |latest_release_date, video| {
                if let Some(release_date) = video.release_date {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::min(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        *[latest_in_playlists, latest_in_subcollections, latest_in_videos]
            .iter()
            .min()
            .unwrap()
    }

    pub fn new(path: String) -> Collection {
        Collection {
            banner: None,
            description: None,
            errors: Vec::new(),
            links: Vec::new(),
            path,
            playlists: Vec::new(),
            subcollections: Vec::new(),
            title: None,
            unlisted: false,
            videos: Vec::new()
        }
    }

    pub fn public_playlists(&self) -> impl Iterator<Item = &Arc<Playlist>> {
        self.playlists.iter().filter(|playlist| !playlist.unlisted)
    }

    pub fn public_playlists_desc_by_release_date(&self) -> Vec<Arc<Playlist>> {
        let mut playlists: Vec<Arc<Playlist>> = self.public_playlists().cloned().collect();
        playlists.sort_unstable_by(|a, b| a.latest_release_date().cmp(&b.latest_release_date()));
        playlists
    }

    pub fn public_subcollections(&self) -> impl Iterator<Item = &Arc<Collection>> {
        self.subcollections.iter().filter(|subcollection| !subcollection.unlisted)
    }

    pub fn public_subcollections_desc_by_release_date(&self) -> Vec<Arc<Collection>> {
        let mut subcollections: Vec<Arc<Collection>> = self.public_subcollections().cloned().collect();
        subcollections.sort_unstable_by(|a, b| a.latest_release_date().cmp(&b.latest_release_date()));
        subcollections
    }

    pub fn public_videos(&self) -> impl Iterator<Item = &Arc<Video>> {
        self.videos.iter().filter(|video| !video.unlisted)
    }

    pub fn public_videos_desc_by_release_date(&self) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.public_videos().cloned().collect();
        videos.sort_unstable_by(|a, b| a.release_date.cmp(&b.release_date));
        videos
    }

    pub fn read_dir(context: &Context, hyper_dir: &HyperDir, site: &Site) -> Arc<Collection> {
        let mut collection = Collection::new(hyper_dir.site_path(context));

        if let Some(manifest) = &hyper_dir.collection_manifest {
            collection.apply_manifest(context, manifest);
        }

        for subdir in &hyper_dir.dirs {
            let hyper_subdir = HyperDir::read(subdir);
            collection.read_subdir(context, &hyper_subdir, site);
        }

        // TODO: Some strategy what we do with multiple images (e.g. heuristic* to pick most probable banner, use additional ones, etc.)
        //       (*) largest image? image that best fits the aspect ratio? name convention? etc.
        if !hyper_dir.image_files.is_empty() && collection.banner.is_none() {
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let assets = context.cache.lock().unwrap().fetch_poster_assets(
                &file_meta,
                &file_name,
                &collection.path,
                0.0
            );

            let banner = Banner::new(assets, None, file_meta, file_name);

            collection.banner = Some(banner);
        }

        if !hyper_dir.video_files.is_empty() {
            let mut error = "No video files should be directly in a collection directory, maybe you wanted to put them in a subfolder instead?".to_string();

            for video_file in &hyper_dir.video_files {
                error.push_str(&format!("\n- {}", video_file.display()));
            }

            collection.errors.push(error);
        }

        if !hyper_dir.errors.is_empty() {
            let mut error = "Directories and/or files inside this collection directory have issues:".to_string();

            for message in &hyper_dir.errors {
                error.push_str(&format!("\n- {}", message));
            }

            collection.errors.push(error);
        }

        Arc::new(collection)
    }

    fn read_subdir(&mut self, context: &Context, hyper_subdir: &HyperDir, site: &Site) {
        if hyper_subdir.collection_manifest.is_some() {
            if hyper_subdir.playlist_manifest.is_some() || hyper_subdir.video_manifest.is_some() {
                let error = format!("Ignoring collection subdirectory {} because it contains conflicting manifest types (only one of collection.eno, playlist.eno and video.eno may exist in one directory)", hyper_subdir.path.display());
                self.errors.push(error);
            } else {
                let subcollection = Collection::read_dir(context, hyper_subdir, site);
                self.subcollections.push(subcollection);
            }
        } else if hyper_subdir.playlist_manifest.is_some() {
            if hyper_subdir.video_manifest.is_some() {
                let error = format!("Ignoring collection subdirectory {} because it contains both playlist.eno and video.eno (only one of the two may exist in one directory)", hyper_subdir.path.display());
                self.errors.push(error);
            } else {
                let playlist = Playlist::read_dir(context, hyper_subdir, site);
                self.playlists.push(playlist);
            }
        } else if hyper_subdir.video_manifest.is_some() {
            let video = Video::read_dir(context, hyper_subdir, site);
            self.videos.push(video);
        } else if !hyper_subdir.dirs.is_empty() {
            let subcollection = Collection::read_dir(context, hyper_subdir, site);
            self.subcollections.push(subcollection);
        } else if !hyper_subdir.video_files.is_empty() {
            let video = Video::read_dir(context, hyper_subdir, site);
            self.videos.push(video);
        } else {
            // TODO: We currently ignore an empty subdir, as if it didn't exist.
            //       Should we rather register it somehow (e.g. implicity as collection),
            //       or handle it as an "Empty" node as we do in SiteContent::Empty?, in
            //       order to communicate the existence via the brower interface too.
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
            let manifest_path = context.site_dir.join("collection.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(banner) = &self.banner {
                let banner_path = context.site_dir.join(&banner.file_name);
                let _ = fs::remove_file(&banner_path);
            }

            for playlist in &self.playlists {
                let playlist_path = context.site_dir.join(playlist.slug());
                let _ = checked_remove_dir_all(context, &playlist_path);
            }

            for subcollection in &self.subcollections {
                let subcollection_path = context.site_dir.join(subcollection.slug());
                let _ = checked_remove_dir_all(context, &subcollection_path);
            }

            for video in &self.videos {
                let video_path = context.site_dir.join(video.slug());
                let _ = checked_remove_dir_all(context, &video_path);
            }
        } else {
            let collection_dir = context.site_dir.join(&self.path);
            let _ = checked_remove_dir_all(context, &collection_dir);
        }
    }

    pub fn update_collection(
        &mut self,
        mutation: impl FnOnce(&mut Collection),
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Arc<Collection>, ()> {
        if let Some(slug) = path_parts.next() {
            if path_parts.peek().is_some() {
                if let Some(subcollection) = self.subcollections
                    .iter_mut()
                    .find(|subcollection| subcollection.slug() == slug) {
                    let mut subcollection_mut = (**subcollection).clone();

                    if let Ok(subcollection_new) = subcollection_mut.update_collection(mutation, path_parts) {
                        *subcollection = Arc::new(subcollection_mut);
                        return Ok(subcollection_new);
                    }
                }
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();
                mutation(&mut subcollection_mut);
                *subcollection = Arc::new(subcollection_mut);
                return Ok(Arc::clone(subcollection));
            }
        }

        Err(())
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
            // Here we know the update occured "deeper down", so we 
            // (mostly) just clone the tree nodes at this level and
            // delegate actual reads/updates to the lower leaves of the tree.

            if let Some(index) = self.videos
                .iter()
                .position(|video| video.slug() == slug) {
                // A video isn't allowed to have subfolders, so this is a bit of an odd case,
                // so we just remove and replace it with whatever happened down this branch.
                self.videos.swap_remove(index);
                let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
                self.read_subdir(context, &hyper_subdir, site);
            } else if let Some(playlist) = self.playlists
                .iter_mut()
                .find(|playlist| playlist.slug() == slug) {
                let mut playlist_mut = (**playlist).clone();
                playlist_mut.update_from_filesystem(context, path_parts, site);
                *playlist = Arc::new(playlist_mut);
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();
                subcollection_mut.update_from_filesystem(context, path_parts, site);
                *subcollection = Arc::new(subcollection_mut);
            } else {
                // Here a sub-subdir needs to be updated (e.g. a video in a
                // collection within this collection) that we haven't
                // even yet registered as subdir, so we read the entire
                // subtree.
                let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
                self.read_subdir(context, &hyper_subdir, site);
            }
        } else {
            // If we presently have a video, playlist or collection registered at the given path, we remove it.
            if let Some(index) = self.videos
                .iter()
                .position(|video| video.slug() == slug) {
                self.videos.swap_remove(index);
            } else if let Some(index) = self.playlists
                .iter()
                .position(|playlist| playlist.slug() == slug) {
                self.playlists.swap_remove(index);
            } else if let Some(index) = self.subcollections
                .iter()
                .position(|subcollection| subcollection.slug() == slug) {
                self.subcollections.swap_remove(index);
            }

            // Add the new leaf (respectively, replace what was removed)
            let hyper_subdir = HyperDir::read(&context.site_dir.join(&self.path).join(slug));
            self.read_subdir(context, &hyper_subdir, site);
        }
    }

    pub fn update_playlist(
        &mut self,
        mutation: impl FnOnce(&mut Playlist),
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Arc<Playlist>, ()> {
        if let Some(slug) = path_parts.next() {
            if let Some(playlist) = self.playlists
                .iter_mut()
                .find(|playlist| playlist.slug() == slug) {
                let mut playlist_mut = (**playlist).clone();
                mutation(&mut playlist_mut);
                *playlist = Arc::new(playlist_mut);
                return Ok(Arc::clone(playlist));
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();

                if let Ok(playlist) = subcollection_mut.update_playlist(mutation, path_parts) {
                    *subcollection = Arc::new(subcollection_mut);
                    return Ok(playlist);
                }
            }
        }

        Err(())
    }

    pub fn update_video(
        &mut self,
        mutation: impl FnOnce(&mut Video),
        mut path_parts: Peekable<Split<char>>
    ) -> Result<Arc<Video>, ()> {
        if let Some(slug) = path_parts.next() {
            if let Some(video) = self.videos
                .iter_mut()
                .find(|video| video.slug() == slug) {
                let mut video_mut = (**video).clone();
                mutation(&mut video_mut);
                *video = Arc::new(video_mut);
                return Ok(Arc::clone(video));
            } else if let Some(playlist) = self.playlists
                .iter_mut()
                .find(|playlist| playlist.slug() == slug) {
                let mut playlist_mut = (**playlist).clone();

                if let Ok(video) = playlist_mut.update_video(mutation, path_parts) {
                    *playlist = Arc::new(playlist_mut);
                    return Ok(video);
                }
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| subcollection.slug() == slug) {
                let mut subcollection_mut = (**subcollection).clone();

                if let Ok(video) = subcollection_mut.update_video(mutation, path_parts) {
                    *subcollection = Arc::new(subcollection_mut);
                    return Ok(video);
                }
            }
        }

        Err(())
    }

    pub fn write_manifest(&self, context: &Context) {
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

        for link in &self.links {
            eno.push_str(&link.to_eno());
        }

        if let Some(description) = &self.description {
            if !eno.is_empty() { eno.push('\n'); }
            eno.push_str(&format!("-- description\n{}\n--description\n", description));
        }

        let path = context.site_dir.join(&self.path).join("collection.eno");
        fs::write(path, eno).unwrap();
    }
}

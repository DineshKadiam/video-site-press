use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_derive::Deserialize;
use slug::slugify;

use crate::{
    Cache,
    Collection,
    EditorTranslations,
    HyperDir,
    Job,
    JobKind,
    Playlist,
    Processing,
    Site,
    SiteContent,
    Theme,
    Video,
    VideoParent
};
use crate::util::nanoid;

pub struct Context {
    pub build_dir: PathBuf,
    pub cache: Mutex<Cache>,
    pub cache_dir: PathBuf,
    // TODO: This is currently statically set to true
    //       Make this configurable, and possibly at runtime.
    //       (in which case it needs to go e.g. in/on site,
    //       because context is immutable (with interior mutability on some things))
    pub clean_urls: bool,
    pub editor_theme: Theme,
    pub editor_translations: EditorTranslations,
    pub job_queue: Mutex<Vec<Job>>,
    pub processing: Mutex<Processing>,
    pub processing_errors: Mutex<Vec<String>>,
    pub site: Mutex<Arc<Site>>,
    pub site_dir: PathBuf,
}

#[derive(Deserialize)]
pub struct ContextPath {
    pub path: String
}

pub enum GetAnyResult {
    Collection(Arc<Collection>),
    None,
    Playlist(Arc<Playlist>),
    Video(Arc<Video>)
}

impl Context {
    pub fn complete_job(&self, id: &str) {
        let mut job_queue = self.job_queue.lock().unwrap();

        job_queue
            .iter()
            .position(|job| job.id == id)
            .map(|index| job_queue.swap_remove(index));
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

    pub fn compute_job_queue(&self) {
        let site = self.get_site();

        let mut job_kinds = Vec::new();

        match &site.content {
            SiteContent::Collection(collection) => self.queue_collection_jobs(collection, &site, &mut job_kinds),
            SiteContent::Playlist(playlist) => self.queue_playlist_jobs(playlist, &site, &mut job_kinds),
            SiteContent::Video(video) => self.queue_video_jobs(video, &site, &mut job_kinds),
            _ => ()
        }

        if !job_kinds.is_empty() {
            self.extend_job_queue(job_kinds);
        }
    }

    pub fn extend_job_queue(&self, job_kinds: Vec<JobKind>) {
        if let Ok(mut job_queue) = self.job_queue.lock() {
            for job_kind in job_kinds {
                if !job_queue
                    .iter()
                    .any(|existing_job| existing_job.kind == job_kind) {
                    job_queue.push(Job::new(job_kind));
                }
            }
        }
    }

    pub fn push_job_queue(&self, job_kind: JobKind) {
        if let Ok(mut job_queue) = self.job_queue.lock() {
            if !job_queue
                .iter()
                .any(|existing_job| existing_job.kind == job_kind) {
                job_queue.push(Job::new(job_kind));
            }
        }
    }

    pub fn convert_to_collection(&self, path: &str) -> Result<(), String> {
        let mut site_mut = self.get_site_mut();

        // TODO: Implement conversion at any level in the site tree
        if path != "" {
            return Err(String::from("Unimplemented: Converting to collection/playlist/video is for now only possible at the root level of the site"));
        }

        if let SiteContent::Empty = &site_mut.content {
            let hyper_dir = HyperDir::read(&self.site_dir);
            let collection = Collection::read_dir(self, &hyper_dir, &site_mut);
            collection.write_manifest(self);
            site_mut.content = SiteContent::Collection(collection);
            self.replace_site(site_mut);
        } else if let SiteContent::Playlist(playlist) = &site_mut.content {
            let manifest_path = self.site_dir.join("playlist.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(banner) = &playlist.banner {
                let banner_path = self.site_dir.join(&banner.file_name);
                let _ = fs::remove_file(&banner_path);
            }

            let hyper_dir = HyperDir::read(&self.site_dir);
            let mut collection = Collection::read_dir(self, &hyper_dir, &site_mut);

            // This always succeeds, as the Arc is guaranteed single-owner at this point.
            if let Some(collection) = Arc::<Collection>::get_mut(&mut collection) {
                if let Some(description) = &playlist.description {
                    collection.description = Some(description.clone());
                }
                if let Some(title) = &playlist.title {
                    collection.title = Some(title.clone());
                }
            };

            collection.write_manifest(self);
            site_mut.content = SiteContent::Collection(collection);
            self.replace_site(site_mut);
        } else if let SiteContent::Video(video) = &site_mut.content {
            let slug = match &video.title {
                Some(title) => slugify(title),
                None => match video.highest_resolution_version() {
                    Some(version) => {
                        Path::new(&version.file_name).file_stem().unwrap().to_string_lossy().to_string()
                    }
                    None => String::from("video")
                }
            };

            let video_dir = self.site_dir.join(&slug);
            fs::create_dir(&video_dir).unwrap();

            if let Ok(dir_entries) = self.site_dir.read_dir() {
                for dir_entry_result in dir_entries {
                    if let Ok(dir_entry) = dir_entry_result {
                        // Skip the video dir itself, and any hidden files/dirs (e.g. cache/build dir!)
                        if dir_entry.file_name().to_string_lossy().starts_with('.') ||
                            dir_entry.path() == video_dir {
                            continue;
                        }

                        let relocation_target_in_video_dir = video_dir.join(dir_entry.file_name());
                        fs::rename(dir_entry.path(), &relocation_target_in_video_dir).unwrap();
                    }
                }
            }

            let hyper_dir = HyperDir::read(&self.site_dir);
            let collection = Collection::read_dir(self, &hyper_dir, &site_mut);

            collection.write_manifest(self);
            site_mut.content = SiteContent::Collection(collection);
            self.replace_site(site_mut);
        }

        Ok(())
    }

    // TODO: Implement this functionality at deeper levels as well, e.g. a
    //       subcollection can also be converted to a playlist, not just the
    //       collection at the root of the site. Same for convert_to_*.
    pub fn convert_to_playlist(&self, path: &str) -> Result<(), String> {
        let mut site_mut = self.get_site_mut();

        // TODO: Implement conversion at any level in the site tree
        if path != "" {
            return Err(String::from("Unimplemented: Converting to collection/playlist/video is for now only possible at the root level of the site"));
        }

        if let SiteContent::Empty = &site_mut.content {
            let hyper_dir = HyperDir::read(&self.site_dir);
            let playlist = Playlist::read_dir(self, &hyper_dir, &site_mut);
            playlist.write_manifest(&self.site_dir);
            site_mut.content = SiteContent::Playlist(playlist);
            self.replace_site(site_mut);
        } else if let SiteContent::Collection(collection) = &site_mut.content {
            if !collection.playlists.is_empty() {
                return Err("A collection containing (a) playlist(s) can not be converted to a playlist".to_string());
            } else if !collection.subcollections.is_empty() {
                return Err("A collection containing (a) subcollection(s) can not be converted to a playlist".to_string());
            }

            let manifest_path = self.site_dir.join("collection.eno");
            let _ = fs::remove_file(&manifest_path);

            let hyper_dir = HyperDir::read(&self.site_dir);
            let mut playlist = Playlist::read_dir(self, &hyper_dir, &site_mut);

            // This always succeeds, as the Arc is guaranteed single-owner at this point.
            if let Some(playlist) = Arc::<Playlist>::get_mut(&mut playlist) {
                if let Some(description) = &collection.description {
                    playlist.description = Some(description.clone());
                }
                if let Some(title) = &collection.title {
                    playlist.title = Some(title.clone());
                }
            };

            playlist.write_manifest(&self.site_dir);
            site_mut.content = SiteContent::Playlist(playlist);
            self.replace_site(site_mut);
        } else if let SiteContent::Video(video) = &site_mut.content {
            let slug = match &video.title {
                Some(title) => slugify(title),
                None => match video.highest_resolution_version() {
                    Some(version) => {
                        Path::new(&version.file_name).file_stem().unwrap().to_string_lossy().to_string()
                    }
                    None => String::from("video")
                }
            };

            let video_dir = self.site_dir.join(&slug);
            fs::create_dir(&video_dir).unwrap();

            if let Ok(dir_entries) = self.site_dir.read_dir() {
                for dir_entry_result in dir_entries {
                    if let Ok(dir_entry) = dir_entry_result {
                        // Skip the video dir itself, and any hidden files/dirs (e.g. cache/build dir!)
                        if dir_entry.file_name().to_string_lossy().starts_with('.') ||
                            dir_entry.path() == video_dir {
                            continue;
                        }

                        let relocation_target_in_video_dir = video_dir.join(dir_entry.file_name());
                        fs::rename(dir_entry.path(), &relocation_target_in_video_dir).unwrap();
                    }
                }
            }

            let hyper_dir = HyperDir::read(&self.site_dir);
            let playlist = Playlist::read_dir(self, &hyper_dir, &site_mut);

            playlist.write_manifest(&self.site_dir);
            site_mut.content = SiteContent::Playlist(playlist);
            self.replace_site(site_mut);
        } 

        Ok(())
    }

    pub fn convert_to_video(&self, path: &str) -> Result<(), String> {
        let mut site_mut = self.get_site_mut();

        // TODO: Implement conversion at any level in the site tree
        if path != "" {
            return Err(String::from("Unimplemented: Converting to collection/playlist/video is for now only possible at the root level of the site"));
        }

        if let SiteContent::Empty = &site_mut.content {
            let hyper_dir = HyperDir::read(&self.site_dir);
            let video = Video::read_dir(self, &hyper_dir, &site_mut);
            video.write_manifest(self);
            site_mut.content = SiteContent::Video(video);
            self.replace_site(site_mut);
        } else if let SiteContent::Collection(collection) = &site_mut.content {
            if !collection.playlists.is_empty() {
                return Err("A collection containing (a) playlist(s) can not be converted to a single video".to_string());
            } else if !collection.subcollections.is_empty() {
                return Err("A collection containing (a) subcollection(s) can not be converted to a single video".to_string());
            } else if collection.videos.len() > 2 {
                return Err("A collection containing two or more videos can not be converted to a single video".to_string());
            }

            let manifest_path = self.site_dir.join("collection.eno");
            let _ = fs::remove_file(&manifest_path);

            // If a (single) video dir exists in the collection, move every file/dir in it into the root,
            // then delete it.
            if let Some(video) = collection.videos.first() {
                let temp_dir = self.site_dir.join(nanoid());

                // Rename the video directory to unique temporary name, so we can copy
                // files/folders within without any possible name collisions with the
                // video directory name itself.
                fs::rename(self.site_dir.join(&video.path), &temp_dir).unwrap();

                if let Ok(dir_entries) = temp_dir.read_dir() {
                    for dir_entry_result in dir_entries {
                        if let Ok(dir_entry) = dir_entry_result {
                            let relocation_target_in_parent = self.site_dir.join(dir_entry.file_name());

                            // If a file/dir with the same name exists in the parent directory, we delete the
                            // one in the parent directory.
                            if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                if metadata.file_type().is_dir() {
                                    fs::remove_dir_all(&relocation_target_in_parent).unwrap();
                                } else {
                                    fs::remove_file(&relocation_target_in_parent).unwrap();
                                }
                            }

                            fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                        }
                    }
                }

                let _ = fs::remove_dir_all(&temp_dir);
            }

            let hyper_dir = HyperDir::read(&self.site_dir);
            let mut video = Video::read_dir(self, &hyper_dir, &site_mut);

            if collection.videos.is_empty() {
                // This always succeeds, as the Arc is guaranteed single-owner at this point.
                if let Some(video) = Arc::<Video>::get_mut(&mut video) {
                    if let Some(description) = &collection.description {
                        video.description = Some(description.clone());
                    }
                    if let Some(title) = &collection.title {
                        video.title = Some(title.clone());
                    }
                };

            }

            video.write_manifest(self);

            site_mut.content = SiteContent::Video(video);
            self.replace_site(site_mut);
        } else if let SiteContent::Playlist(playlist) = &site_mut.content {
            if playlist.videos.len() > 1 {
                return Err("A playlist containing two or more videos can not be converted to a single video".to_string());
            }

            let manifest_path = self.site_dir.join("playlist.eno");
            let _ = fs::remove_file(&manifest_path);

            // If a (single) video dir exists in the playlist, move every file/dir in it into the root,
            // then delete it.
            if let Some(video) = playlist.videos.first() {
                let temp_dir = self.site_dir.join(nanoid());

                // Rename the video directory to unique temporary name, so we can copy
                // files/folders within without any possible name collisions with the
                // video directory name itself.
                fs::rename(self.site_dir.join(&video.path), &temp_dir).unwrap();

                if let Ok(dir_entries) = temp_dir.read_dir() {
                    for dir_entry_result in dir_entries {
                        if let Ok(dir_entry) = dir_entry_result {
                            let relocation_target_in_parent = self.site_dir.join(dir_entry.file_name());

                            // If a file/dir with the same name exists in the parent directory, we delete the
                            // one in the parent directory.
                            if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                if metadata.file_type().is_dir() {
                                    fs::remove_dir_all(&relocation_target_in_parent).unwrap();
                                } else {
                                    fs::remove_file(&relocation_target_in_parent).unwrap();
                                }
                            }

                            fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                        }
                    }
                }

                let _ = fs::remove_dir_all(&temp_dir);
            }

            let hyper_dir = HyperDir::read(&self.site_dir);
            let mut video = Video::read_dir(self, &hyper_dir, &site_mut);

            if playlist.videos.is_empty() {
                // This always succeeds, as the Arc is guaranteed single-owner at this point.
                if let Some(video) = Arc::<Video>::get_mut(&mut video) {
                    if let Some(description) = &playlist.description {
                        video.description = Some(description.clone());
                    }
                    if let Some(title) = &playlist.title {
                        video.title = Some(title.clone());
                    }
                };

            }

            video.write_manifest(self);

            site_mut.content = SiteContent::Video(video);
            self.replace_site(site_mut);
        }

        Ok(())
    }

    pub fn delete_collection(
        &self,
        path: &str
    ) -> bool {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Collection(collection) = &site_mut.content {
                    collection.unpersist(self);
                    site_mut.content = SiteContent::Empty;
                    self.replace_site(site_mut);
                    return true;
                }
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if collection_mut.delete_collection(self, path_parts) {
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
                return true;
            }
        }

        false
    }

    pub fn delete_playlist(
        &self,
        path: &str
    ) -> bool {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Playlist(playlist) = &site_mut.content {
                    playlist.unpersist(self);
                    site_mut.content = SiteContent::Empty;
                    self.replace_site(site_mut);
                    return true;
                }
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if collection_mut.delete_playlist(self, path_parts) {
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
                return true;
            }
        }

        false
    }

    /// Deletes the video at path (if found) and in case of success returns
    // its parent (collection or playlist or none if it was at the site root).
    pub fn delete_video(
        &self,
        path: &str
    ) -> Result<Option<VideoParent>, ()> {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Video(video) = &site_mut.content {
                    video.unpersist(self);
                    site_mut.content = SiteContent::Empty;
                    self.replace_site(site_mut);
                    return Ok(None);
                }
            }
        } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
            let mut playlist_mut = (**playlist).clone();

            if playlist_mut.delete_video(self, path_parts) {
                let playlist_new = Arc::new(playlist_mut);
                *playlist = Arc::clone(&playlist_new);
                self.replace_site(site_mut);
                return Ok(Some(VideoParent::Playlist(playlist_new)));
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(video_parent) = collection_mut.delete_video(self, path_parts) {
                match video_parent {
                    Some(video_parent) => {
                        *collection = Arc::new(collection_mut);
                        self.replace_site(site_mut);
                        return Ok(Some(video_parent));
                    }
                    None => {
                        let collection_new = Arc::new(collection_mut);
                        *collection = Arc::clone(&collection_new);
                        self.replace_site(site_mut);
                        return Ok(Some(VideoParent::Collection(collection_new)))
                    }
                }
            }
        }

        Err(())
    }

    /// Returns the collection, playlist or video at the given path.
    pub fn get_any(&self, path: &str) -> GetAnyResult {
        self.site.lock().unwrap().get_any(path)
    }

    /// Returns the collection at the given path.
    pub fn get_collection(&self, path: &str) -> Option<Arc<Collection>> {
        self.site.lock().unwrap().get_collection(path)
    }

    /// Returns the playlist at the given path.
    pub fn get_playlist(&self, path: &str) -> Option<Arc<Playlist>> {
        self.site.lock().unwrap().get_playlist(path)
    }

    /// Returns a cloned `Arc<Site>`, allowing subsequent read-only processing
    /// of the site tree without keeping a lock on it.
    pub fn get_site(&self) -> Arc<Site> {
        Arc::clone(&*self.site.lock().unwrap())
    }

    /// Returns a mutable `Site` clone of the site tree root, allowing to
    /// build an updated version of the site tree (without keeping a lock on
    /// it in the meanwhile) that can later be commited by replacing the
    /// `Arc<Site>` contained within `Context`.
    pub fn get_site_mut(&self) -> Site {
        (**(self.site.lock().unwrap())).clone()
    }

    /// Returns the video at the given path.
    pub fn get_video(&self, path: &str) -> Option<Arc<Video>> {
        self.site.lock().unwrap().get_video(path)
    }

    pub fn new(
        build_dir: PathBuf,
        cache: Mutex<Cache>,
        cache_dir: PathBuf,
        editor_language: &str,
        site: Arc<Site>,
        site_dir: PathBuf
    ) -> Context {
        Context {
            build_dir,
            cache,
            cache_dir,
            clean_urls: true,
            // TODO: Eventually make this customizable via CLI option
            editor_theme: Theme::adaptive(),
            editor_translations: EditorTranslations::from_code(editor_language),
            job_queue: Mutex::new(Vec::new()),
            processing: Mutex::new(Processing::new()),
            processing_errors: Mutex::new(Vec::new()),
            site: Mutex::new(site),
            site_dir,
        }
    }

    fn queue_collection_jobs(&self, collection: &Arc<Collection>, site: &Arc<Site>, job_kinds: &mut Vec<JobKind>) {
        for subcollection in &collection.subcollections {
            self.queue_collection_jobs(subcollection, site, job_kinds);
        }
        for playlist in &collection.playlists {
            self.queue_playlist_jobs(playlist, site, job_kinds);
        }
        for video in &collection.videos {
            self.queue_video_jobs(video, site, job_kinds);
        }
    }

    fn queue_playlist_jobs(&self, playlist: &Arc<Playlist>, site: &Arc<Site>, job_kinds: &mut Vec<JobKind>) {
        if playlist.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            job_kinds.push(JobKind::PlaylistFastTasks(Arc::clone(playlist)));
        }

        for video in &playlist.videos {
            self.queue_video_jobs(video, site, job_kinds);
        }
    }

    // fn queue_version_jobs(&self, video: &Arc<Video>, queue: &mut Vec<Job>) {
    //     let metadata = video.assets.metadata.as_ref().unwrap();

    //     let settings = self.collection.get_settings();

    //     let mut target_formats = Vec::new();

    //     for format in &settings.formats {
    //         match format {
    //             // TODO: This would actually make more sense as a singleton prop?
    //             //       May have settings, but only one audio format is probably enough.
    //             VideoFormat::AudioOnly => {
    //                 if metadata.has_audio {
    //                     target_formats.push(VideoFormat::AudioOnly);
    //                 }
    //             }
    //             VideoFormat::Video { width, .. } => {
    //                 if metadata.width >= *width {
    //                     target_formats.push(format.clone());
    //                 } 
    //             }
    //         }
    //     }

    //     // TODO: If the video is smaller than the smallest video format
    //     //       we don't get any single video version - how do we model
    //     //       our format list differently so we can nicely prevent
    //     //       even getting in that kind of a problem space?
    //     //       (Should the formats be more kind of suggestions/hints?)

    //     let version_jobs = target_formats
    //         .into_iter()
    //         .filter(|format| !video.has_format(format))
    //         .map(|format| Job::new(JobKind::Format(format), video.clone()));

    //     queue.extend(version_jobs);
    // }

    /// Determine what needs to be computed for this video and append the
    /// required jobs to the passed queue.
    fn queue_video_jobs(&self, video: &Arc<Video>, site: &Arc<Site>, job_kinds: &mut Vec<JobKind>) {
        if video.versions.iter().any(|version| version.video_meta.is_none()) ||
           (video.poster.is_none() && !video.versions.is_empty()) ||
           video.poster.as_ref().is_some_and(|poster|
                poster.assets.is_none() || poster.assets.as_ref().is_some_and(|assets|
                    assets.poster_aspect != site.poster_aspect.numeric()
                )
            ) {
            let job_kind = JobKind::VideoFastTasks {
                poster_aspect: site.poster_aspect.numeric(),
                video: Arc::clone(video)
            };

            job_kinds.push(job_kind);
        }

        // if video.assets.metadata.is_some() {
        //     // TODO: Reactivate, this time needs to be based on markers in the dir/manifest
        //     //       that request a certain audio-only/resolution version?
        //     // self.create_version_jobs(video);
        // }
    }

    /// Replace the immutable `Arc<Site>` at the site tree root,
    /// updating the site tree to a new version.
    pub fn replace_site(&self, site_new: Site) {
        *self.site.lock().unwrap() = Arc::new(site_new);
    }

    /// Take the job with the highest priority that isn't assigned yet.
    /// (The job is left in the queue too, marked as assigned for others)
    pub fn take_processing_job(&self, high_priority_only: bool) -> Option<Job> {
        let mut job_queue = self.job_queue.lock().unwrap();

        job_queue
            .iter_mut()
            .filter(|job| {
                if high_priority_only {
                    if let JobKind::VideoFormat { .. } = job.kind {
                        return false;
                    }
                }

                if job.assigned { return false; }

                true
            })
            .reduce(|selected, iterated|
                match iterated.cmp_priority(selected) { 
                    Ordering::Greater => iterated,
                    _ => selected
                } 
            )
            .map(|job| {
                job.assigned = true;
                job.clone()
            })
    }

    pub fn update_collection(
        &self,
        mutation: impl FnOnce(&mut Collection),
        path: &str
    ) -> Result<Arc<Collection>, ()> {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        let mut collection_new = None;

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Collection(collection) = &mut site_mut.content {
                    let mut collection_mut = (**collection).clone();
                    mutation(&mut collection_mut);
                    *collection = Arc::new(collection_mut);
                    collection_new = Some(Arc::clone(collection));
                }
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(collection_updated) = collection_mut.update_collection(mutation, path_parts) {
                *collection = Arc::new(collection_mut);
                collection_new = Some(collection_updated);
            }
        }

        if let Some(collection) = collection_new {
            self.replace_site(site_mut);
            return Ok(collection);
        }

        Err(())
    }

    pub fn update_from_filesystem(&self, path: &Path, site: &Site) {
        let dir_relative = if path.is_dir() {
            path.strip_prefix(&self.site_dir).unwrap()
        } else if path.is_file() {
            path.parent().unwrap().strip_prefix(&self.site_dir).unwrap()
        } else {
            return;
        };

        let mut path_parts = dir_relative.to_str().unwrap().split('/').peekable();

        if let Some(&"") = path_parts.peek() {
            self.replace_site(Site::read_dir(self, &self.site_dir));
        } else {
            let mut site_mut = self.get_site_mut();

            if let SiteContent::Collection(collection) = &mut site_mut.content {
                let mut collection_mut = (**collection).clone();
                collection_mut.update_from_filesystem(self, path_parts, site);
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
            } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
                let mut playlist_mut = (**playlist).clone();
                playlist_mut.update_from_filesystem(self, path_parts, site);
                *playlist = Arc::new(playlist_mut);
                self.replace_site(site_mut);
            } else if let SiteContent::Video(_) = &mut site_mut.content {
                // This branch is somewhat an odd case, e.g. the site directory
                // had a video manifest in it, but additionally folders (which
                // is not intended by hyper 8 at all), and then the video manifest
                // is moved into one of those folders, triggering an update at a
                // subdirectory that was so far ignored.
                self.replace_site(Site::read_dir(self, &self.site_dir));
            }
        }
    }

    pub fn update_playlist(
        &self,
        mutation: impl FnOnce(&mut Playlist),
        path: &str
    ) -> Result<Arc<Playlist>, ()> {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        let mut playlist_new = None;

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Playlist(playlist) = &mut site_mut.content {
                    let mut playlist_mut = (**playlist).clone();
                    mutation(&mut playlist_mut);
                    *playlist = Arc::new(playlist_mut);
                    playlist_new = Some(Arc::clone(playlist));
                }
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(playlist) = collection_mut.update_playlist(mutation, path_parts) {
                *collection = Arc::new(collection_mut);
                playlist_new = Some(playlist);
            }
        }

        if let Some(playlist) = playlist_new {
            self.replace_site(site_mut);
            return Ok(playlist);
        }

        Err(())
    }

    pub fn update_site(
        &self,
        mutation: impl FnOnce(&mut Site),
    ) -> Arc<Site> {
        let mut site_mut = self.get_site_mut();

        mutation(&mut site_mut);

        let site_new = Arc::new(site_mut);

        *self.site.lock().unwrap() = Arc::clone(&site_new);

        site_new
    }

    pub fn update_video(
        &self,
        mutation: impl FnOnce(&mut Video),
        path: &str
    ) -> Result<Arc<Video>, ()> {
        let mut path_parts = path.split('/').peekable();

        let mut site_mut = self.get_site_mut();

        let mut video_new = None;

        if let Some(&"") = path_parts.peek() {
            if path_parts.count() == 1 {
                if let SiteContent::Video(video) = &mut site_mut.content {
                    let mut video_mut = (**video).clone();
                    mutation(&mut video_mut);
                    *video = Arc::new(video_mut);
                    video_new = Some(Arc::clone(video));
                }
            }
        } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
            let mut playlist_mut = (**playlist).clone();

            if let Ok(video) = playlist_mut.update_video(mutation, path_parts) {
                *playlist = Arc::new(playlist_mut);
                video_new = Some(video);
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(video) = collection_mut.update_video(mutation, path_parts) {
                *collection = Arc::new(collection_mut);
                video_new = Some(video);
            }
        }

        if let Some(video) = video_new {
            self.replace_site(site_mut);
            return Ok(video);
        }

        Err(())
    }
}

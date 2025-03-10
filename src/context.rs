// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use slug::slugify;

use crate::{
    Cache,
    Collection,
    DeploymentState,
    HyperDir,
    InheritedOptions,
    Job,
    JobKind,
    Language,
    Playlist,
    Processing,
    Site,
    SiteContent,
    SitePath,
    Theme,
    TreeViewOrder,
    Video,
    VideoParent
};
use crate::util::{checked_remove_dir_all, nanoid};

pub struct Context {
    pub build_dir: PathBuf,
    pub cache: Mutex<Cache>,
    pub cache_dir: PathBuf,
    pub deploying: Mutex<DeploymentState>,
    pub editor_language: Mutex<Language>,
    pub editor_theme: Mutex<Theme>,
    pub editor_tree_view_order: Mutex<TreeViewOrder>,
    pub job_queue: Mutex<Vec<Job>>,
    pub jobs_completed: Mutex<Vec<Job>>,
    pub processing: Mutex<Processing>,
    pub processing_errors: Mutex<Vec<String>>,
    pub site: Mutex<Arc<Site>>,
    pub site_dir: PathBuf,
}

pub enum GetAnyResult {
    Collection(Arc<Collection>),
    None,
    Playlist(Arc<Playlist>),
    Video(Arc<Video>)
}

/// An update to a collection happens by cloning the existing collection, running
/// a closure on it that mutates it into the new, updated state and lastly
/// replacing the existing collection in the site tree with our new, updated
/// version. The calling code then receives a reference to the new collection,
/// and if it returned anything in the closure that performed the mutation,
/// it is passed back that value through `optional_return_value`.
pub enum CollectionUpdate<T> {
    NotFound,
    Updated {
        collection_updated: Arc<Collection>,
        optional_return_value: T,
    }
}

/// An update to a playlist happens by cloning the existing playlist, running
/// a closure on it that mutates it into the new, updated state and lastly
/// replacing the existing playlist in the site tree with our new, updated
/// version. The calling code then receives a reference to the new playlist,
/// and if it returned anything in the closure that performed the mutation,
/// it is passed back that value through `optional_return_value`.
pub enum PlaylistUpdate<T> {
    NotFound,
    Updated {
        optional_return_value: T,
        playlist_updated: Arc<Playlist>
    }
}

impl Context {
    pub fn await_job_queue(&self) {
        let mut previous_pending_jobs = 0;
        loop {
            let pending_jobs = self.job_queue.lock().unwrap().len();

            if pending_jobs == 0 { break; }

            if pending_jobs != previous_pending_jobs {
                println!("Waiting for {pending_jobs} pending jobs");
                previous_pending_jobs = pending_jobs;
            }

            thread::sleep(Duration::from_secs(1));
        }
    }

    pub fn complete_job(&self, id: &str) {
        let completed_job = {
            let mut job_queue = self.job_queue.lock().unwrap();

            job_queue
                .iter()
                .position(|job| job.id == id)
                .map(|index| job_queue.swap_remove(index))
        };

        if let Some(mut completed_job) = completed_job {
            completed_job.completed_at = Some(SystemTime::now());
            self.jobs_completed.lock().unwrap().push(completed_job);
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

    /// Take the playlist or video at the given path (or if the site is still empty,
    /// the empty site itself) and convert it to a collection. This never fails.
    pub fn convert_to_collection(&self, site_path: &SitePath) {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Empty = &site_mut.content {
                let hyper_dir = HyperDir::read(&self.site_dir);
                let collection = Collection::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);
                collection.write_manifest(self);
                site_mut.content = SiteContent::Collection(collection);
                self.replace_site(site_mut);
            } else if let SiteContent::Playlist(playlist) = &site_mut.content {
                let manifest_path = self.site_dir.join("playlist.eno");
                let _ = fs::remove_file(&manifest_path);

                let hyper_dir = HyperDir::read(&self.site_dir);
                let mut collection = Collection::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                {
                    let collection_mut = Arc::<Collection>::get_mut(&mut collection).unwrap(); // always succeeds, single-owned at this point

                    if let Some(banner) = &playlist.banner {
                        collection_mut.banner = Some(banner.clone());
                    }
                    if let Some(description) = &playlist.description {
                        collection_mut.description = Some(description.clone());
                    }
                    if !playlist.links.is_empty() {
                        collection_mut.links = playlist.links.clone();
                    }
                    collection_mut.offline_local_set(playlist.offline_local());
                    if let Some(title) = &playlist.title {
                        collection_mut.title = Some(title.clone());
                    }
                    collection_mut.unlisted_local_set(playlist.unlisted_local());
                    collection_mut.video_order = playlist.order;
                }

                collection.write_manifest(self);
                site_mut.content = SiteContent::Collection(collection);
                self.replace_site(site_mut);
            } else if let SiteContent::Video(video) = &site_mut.content {
                let slug = match &video.title {
                    Some(title) => slugify(title),
                    None => match video.highest_quality_version() {
                        Some(version) => {
                            slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
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
                let mut collection = Collection::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                {
                    // TODO: Would be quite elegant if offline+unlisted would in turn be removed from the video
                    let collection_mut = Arc::<Collection>::get_mut(&mut collection).unwrap(); // always succeeds, single-owned at this point
                    collection_mut.offline_local_set(video.offline_local());
                    collection_mut.unlisted_local_set(video.unlisted_local());
                }

                collection.write_manifest(self);
                site_mut.content = SiteContent::Collection(collection);
                self.replace_site(site_mut);
            }
        } else {
            let node = self.get_any(site_path);

            if let GetAnyResult::Playlist(playlist) = node {
                let dir_converting = self.site_dir.join(playlist.site_path.filesystem_relative());

                let manifest_path = dir_converting.join("playlist.eno");
                let _ = fs::remove_file(&manifest_path);

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut collection = Collection::read_dir(self, &hyper_dir, playlist.copy_inherited_options(), &site_mut);

               {
                    let collection_mut = Arc::<Collection>::get_mut(&mut collection).unwrap();  // always succeeds, single-owned at this point

                    if let Some(banner) = &playlist.banner {
                        collection_mut.banner = Some(banner.clone());
                    }
                    if let Some(description) = &playlist.description {
                        collection_mut.description = Some(description.clone());
                    }
                    if !playlist.links.is_empty() {
                        collection_mut.links = playlist.links.clone();
                    }
                    collection_mut.offline_local_set(playlist.offline_local());
                    if let Some(title) = &playlist.title {
                        collection_mut.title = Some(title.clone());
                    }
                    collection_mut.unlisted_local_set(playlist.unlisted_local());
                    collection_mut.video_order = playlist.order;
                };

                collection.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            } else if let GetAnyResult::Video(video) = node {
                let dir_converting = self.site_dir.join(video.site_path.filesystem_relative());

                let slug = match &video.title {
                    Some(title) => slugify(title),
                    None => match video.highest_quality_version() {
                        Some(version) => {
                            slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
                        }
                        None => String::from("video")
                    }
                };

                let new_video_dir = dir_converting.join(&slug);
                fs::create_dir(&new_video_dir).unwrap();

                if let Ok(dir_entries) = dir_converting.read_dir() {
                    for dir_entry_result in dir_entries {
                        if let Ok(dir_entry) = dir_entry_result {
                            // Skip the video dir itself, and any hidden files/dirs (e.g. cache/build dir!)
                            if dir_entry.file_name().to_string_lossy().starts_with('.') ||
                                dir_entry.path() == new_video_dir {
                                continue;
                            }

                            let relocation_target_in_new_video_dir = new_video_dir.join(dir_entry.file_name());
                            fs::rename(dir_entry.path(), &relocation_target_in_new_video_dir).unwrap();
                        }
                    }
                }

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut collection = Collection::read_dir(self, &hyper_dir, video.copy_inherited_options(), &site_mut);

                {
                    // TODO: Would be quite elegant if offline+unlisted would in turn be removed from the video
                    let collection_mut = Arc::<Collection>::get_mut(&mut collection).unwrap(); // always succeeds, single-owned at this point
                    collection_mut.offline_local_set(video.offline_local());
                    collection_mut.unlisted_local_set(video.unlisted_local());
                }

                collection.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            }
        }
    }

    /// Take the collection or video at the given path (or if the site is still empty,
    /// the empty site itself) and convert it to a playlist. This may fail if we're
    /// converting from a collection and that collection happens to contain playlists
    /// or subcollections, which a playlist can not contain.
    pub fn convert_to_playlist(&self, site_path: &SitePath) -> Result<(), String> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Empty = &site_mut.content {
                let hyper_dir = HyperDir::read(&self.site_dir);
                let playlist = Playlist::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);
                playlist.write_manifest(self);
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
                let mut playlist = Playlist::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                {
                    let playlist_mut = Arc::<Playlist>::get_mut(&mut playlist).unwrap(); // always succeeds, single-owned at this point

                    if let Some(banner) = &collection.banner {
                        playlist_mut.banner = Some(banner.clone());
                    }
                    if let Some(description) = &collection.description {
                        playlist_mut.description = Some(description.clone());
                    }
                    if !collection.links.is_empty() {
                        playlist_mut.links = collection.links.clone();
                    }
                    playlist_mut.offline_local_set(collection.offline_local());
                    playlist_mut.order = collection.video_order;
                    if let Some(title) = &collection.title {
                        playlist_mut.title = Some(title.clone());
                    }
                    playlist_mut.unlisted_local_set(collection.unlisted_local());
                }

                playlist.write_manifest(self);
                site_mut.content = SiteContent::Playlist(playlist);
                self.replace_site(site_mut);
            } else if let SiteContent::Video(video) = &site_mut.content {
                let slug = match &video.title {
                    Some(title) => slugify(title),
                    None => match video.highest_quality_version() {
                        Some(version) => {
                            slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
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
                let mut playlist = Playlist::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                {
                    // TODO: Would be quite elegant if offline+unlisted would in turn be removed from the video
                    let playlist_mut = Arc::<Playlist>::get_mut(&mut playlist).unwrap(); // always succeeds, single-owned at this point
                    playlist_mut.offline_local_set(video.offline_local());
                    playlist_mut.unlisted_local_set(video.unlisted_local());
                }

                playlist.write_manifest(self);
                site_mut.content = SiteContent::Playlist(playlist);
                self.replace_site(site_mut);
            } 
        } else {
            let node = self.get_any(site_path);

            if let GetAnyResult::Collection(collection) = node {
                if !collection.playlists.is_empty() {
                    return Err("A collection containing (a) playlist(s) can not be converted to a playlist".to_string());
                } else if !collection.subcollections.is_empty() {
                    return Err("A collection containing (a) subcollection(s) can not be converted to a playlist".to_string());
                }

                let dir_converting = self.site_dir.join(collection.site_path.filesystem_relative());

                let manifest_path = dir_converting.join("collection.eno");
                let _ = fs::remove_file(&manifest_path);

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut playlist = Playlist::read_dir(self, &hyper_dir, collection.copy_inherited_options(), &site_mut);

                {
                    let playlist_mut = Arc::<Playlist>::get_mut(&mut playlist).unwrap();  // always succeeds, single-owned at this point

                    if let Some(banner) = &collection.banner {
                        playlist_mut.banner = Some(banner.clone());
                    }
                    if let Some(description) = &collection.description {
                        playlist_mut.description = Some(description.clone());
                    }
                    if !collection.links.is_empty() {
                        playlist_mut.links = collection.links.clone();
                    }
                    playlist_mut.offline_local_set(collection.offline_local());
                    playlist_mut.order = collection.video_order;
                    if let Some(title) = &collection.title {
                        playlist_mut.title = Some(title.clone());
                    }
                    playlist_mut.unlisted_local_set(collection.unlisted_local());
                }

                playlist.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            } else if let GetAnyResult::Video(video) = node {
                let dir_converting = self.site_dir.join(video.site_path.filesystem_relative());

                let slug = match &video.title {
                    Some(title) => slugify(title),
                    None => match video.highest_quality_version() {
                        Some(version) => {
                            slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
                        }
                        None => String::from("video")
                    }
                };

                let new_video_dir = dir_converting.join(&slug);
                fs::create_dir(&new_video_dir).unwrap();

                if let Ok(dir_entries) = dir_converting.read_dir() {
                    for dir_entry_result in dir_entries {
                        if let Ok(dir_entry) = dir_entry_result {
                            // Skip the video dir itself, and any hidden files/dirs (e.g. cache/build dir!)
                            if dir_entry.file_name().to_string_lossy().starts_with('.') ||
                                dir_entry.path() == new_video_dir {
                                continue;
                            }

                            let relocation_target_in_new_video_dir = new_video_dir.join(dir_entry.file_name());
                            fs::rename(dir_entry.path(), &relocation_target_in_new_video_dir).unwrap();
                        }
                    }
                }

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut playlist = Playlist::read_dir(self, &hyper_dir, video.copy_inherited_options(), &site_mut);

                {
                    // TODO: Would be quite elegant if offline+unlisted would in turn be removed from the video
                    let playlist_mut = Arc::<Playlist>::get_mut(&mut playlist).unwrap(); // always succeeds, single-owned at this point
                    playlist_mut.offline_local_set(video.offline_local());
                    playlist_mut.unlisted_local_set(video.unlisted_local());
                }

                playlist.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            }
        }

        Ok(())
    }

    /// Take the collection or playlist at the given path (or if the site is still empty,
    /// the empty site itself) and convert it to a video. This only succeeds if we're
    /// converting from a collection or playlist that contains exactly one video (or none).
    pub fn convert_to_video(&self, site_path: &SitePath) -> Result<(), String> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Empty = &site_mut.content {
                let hyper_dir = HyperDir::read(&self.site_dir);
                let video = Video::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);
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
                    fs::rename(self.site_dir.join(video.site_path.filesystem_relative()), &temp_dir).unwrap();

                    if let Ok(dir_entries) = temp_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let relocation_target_in_parent = self.site_dir.join(dir_entry.file_name());

                                // If a file/dir with the same name exists in the parent directory, we delete the
                                // one in the parent directory.
                                if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                    if metadata.file_type().is_dir() {
                                        checked_remove_dir_all(self, &relocation_target_in_parent).unwrap();
                                    } else {
                                        fs::remove_file(&relocation_target_in_parent).unwrap();
                                    }
                                }

                                fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                            }
                        }
                    }

                    let _ = checked_remove_dir_all(self, &temp_dir);
                }

                let hyper_dir = HyperDir::read(&self.site_dir);
                let mut video = Video::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                {
                    let video_mut = Arc::<Video>::get_mut(&mut video).unwrap(); // always succeeds, single-owned at this point

                    // We only copy fields from the collection to the new video,
                    // if the video was not initialized from a single video that
                    // already existed in the collection.
                    if collection.videos.is_empty() {
                        // TODO: We could consider copying a collection banner to be the new video poster here
                        if let Some(description) = &collection.description {
                            video_mut.description = Some(description.clone());
                        }
                        if !collection.links.is_empty() {
                            video_mut.links = collection.links.clone();
                        }
                        if let Some(title) = &collection.title {
                            video_mut.title = Some(title.clone());
                        }
                    }

                    // If the collection was offline, we ensure that the video
                    // that was indirectly offline as its child, stays offline
                    // when it replaces the collection
                    if collection.offline_local() && !video_mut.offline_local() {
                        video_mut.offline_local_set(true);
                    }

                    // If the collection was unlisted, we ensure that the video
                    // that was indirectly unlisted as its child, stays unlisted
                    // when it replaces the collection
                    if collection.unlisted_local() && !video_mut.unlisted_local() {
                        video_mut.unlisted_local_set(true);
                    }
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
                    fs::rename(self.site_dir.join(video.site_path.filesystem_relative()), &temp_dir).unwrap();

                    if let Ok(dir_entries) = temp_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let relocation_target_in_parent = self.site_dir.join(dir_entry.file_name());

                                // If a file/dir with the same name exists in the parent directory, we delete the
                                // one in the parent directory.
                                if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                    if metadata.file_type().is_dir() {
                                        checked_remove_dir_all(self, &relocation_target_in_parent).unwrap();
                                    } else {
                                        fs::remove_file(&relocation_target_in_parent).unwrap();
                                    }
                                }

                                fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                            }
                        }
                    }

                    let _ = checked_remove_dir_all(self, &temp_dir);
                }

                let hyper_dir = HyperDir::read(&self.site_dir);
                let mut video = Video::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                // This always succeeds, as the Arc is guaranteed single-owner at this point.
               {
                    let video_mut = Arc::<Video>::get_mut(&mut video).unwrap(); // always succeeds, single-owned at this point

                    // We only copy fields from the playlist to the new video,
                    // if the video was not initialized from a single video that
                    // already existed in the playlist.
                    if playlist.videos.is_empty() {
                        // TODO: We could consider copying a playlist banner to be the new video poster here
                        if let Some(description) = &playlist.description {
                            video_mut.description = Some(description.clone());
                        }
                        if !playlist.links.is_empty() {
                            video_mut.links = playlist.links.clone();
                        }
                        if let Some(title) = &playlist.title {
                            video_mut.title = Some(title.clone());
                        }
                    }

                    // If the playlist was offline, we ensure that the video
                    // that was indirectly offline as its child, stays offline
                    // when it replaces the playlist
                    if playlist.offline_local() && !video_mut.offline_local() {
                        video_mut.offline_local_set(true);
                    }

                    // If the playlist was unlisted, we ensure that the video
                    // that was indirectly unlisted as its child, stays unlisted
                    // when it replaces the playlist
                    if playlist.unlisted_local() && !video_mut.unlisted_local() {
                        video_mut.unlisted_local_set(true);
                    }
                };

                video.write_manifest(self);

                site_mut.content = SiteContent::Video(video);
                self.replace_site(site_mut);
            }
        } else {
            let node = self.get_any(site_path);

            if let GetAnyResult::Collection(collection) = node {
                if !collection.playlists.is_empty() {
                    return Err("A collection containing (a) playlist(s) can not be converted to a single video".to_string());
                } else if !collection.subcollections.is_empty() {
                    return Err("A collection containing (a) subcollection(s) can not be converted to a single video".to_string());
                } else if collection.videos.len() > 2 {
                    return Err("A collection containing two or more videos can not be converted to a single video".to_string());
                }

                let dir_converting = self.site_dir.join(collection.site_path.filesystem_relative());

                let manifest_path = dir_converting.join("collection.eno");
                let _ = fs::remove_file(&manifest_path);

                // If a (single) video dir exists in the collection, move every file/dir in it into the root,
                // then delete it.
                if let Some(video) = collection.videos.first() {
                    let temp_dir = dir_converting.join(nanoid());

                    // Rename the video directory to unique temporary name, so we can copy
                    // files/folders within without any possible name collisions with the
                    // video directory name itself.
                    fs::rename(self.site_dir.join(video.site_path.filesystem_relative()), &temp_dir).unwrap();

                    if let Ok(dir_entries) = temp_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let relocation_target_in_parent = dir_converting.join(dir_entry.file_name());

                                // If a file/dir with the same name exists in the parent directory, we delete the
                                // one in the parent directory.
                                if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                    if metadata.file_type().is_dir() {
                                        checked_remove_dir_all(self, &relocation_target_in_parent).unwrap();
                                    } else {
                                        fs::remove_file(&relocation_target_in_parent).unwrap();
                                    }
                                }

                                fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                            }
                        }
                    }

                    let _ = checked_remove_dir_all(self, &temp_dir);
                }

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut video = Video::read_dir(self, &hyper_dir, collection.copy_inherited_options(), &site_mut);

                {
                    let video_mut = Arc::<Video>::get_mut(&mut video).unwrap(); // always succeeds, single-owned at this point
                    // We only copy fields from the collection to the new video,
                    // if the video was not initialized from a single video that
                    // already existed in the collection.
                    if collection.videos.is_empty() {
                        // TODO: We could consider copying a collection banner to be the new video poster here
                        if let Some(description) = &collection.description {
                            video_mut.description = Some(description.clone());
                        }
                        if !collection.links.is_empty() {
                            video_mut.links = collection.links.clone();
                        }
                        if let Some(title) = &collection.title {
                            video_mut.title = Some(title.clone());
                        }
                    }

                    // If the collection was offline, we ensure that the video
                    // that was indirectly offline as its child, stays offline
                    // when it replaces the collection
                    if collection.offline_local() && !video_mut.offline_local() {
                        video_mut.offline_local_set(true);
                    }

                    // If the collection was unlisted, we ensure that the video
                    // that was indirectly unlisted as its child, stays unlisted
                    // when it replaces the collection
                    if collection.unlisted_local() && !video_mut.unlisted_local() {
                        video_mut.unlisted_local_set(true);
                    }
                }

                video.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            } else if let GetAnyResult::Playlist(playlist) = node {
                if playlist.videos.len() > 1 {
                    return Err("A playlist containing two or more videos can not be converted to a single video".to_string());
                }

                let dir_converting = self.site_dir.join(playlist.site_path.filesystem_relative());

                let manifest_path = dir_converting.join("playlist.eno");
                let _ = fs::remove_file(&manifest_path);

                // If a (single) video dir exists in the playlist, move every file/dir in it into the root,
                // then delete it.
                if let Some(video) = playlist.videos.first() {
                    let temp_dir = dir_converting.join(nanoid());

                    // Rename the video directory to unique temporary name, so we can copy
                    // files/folders within without any possible name collisions with the
                    // video directory name itself.
                    fs::rename(self.site_dir.join(video.site_path.filesystem_relative()), &temp_dir).unwrap();

                    if let Ok(dir_entries) = temp_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let relocation_target_in_parent = dir_converting.join(dir_entry.file_name());

                                // If a file/dir with the same name exists in the parent directory, we delete the
                                // one in the parent directory.
                                if let Ok(metadata) = fs::metadata(&relocation_target_in_parent) {
                                    if metadata.file_type().is_dir() {
                                        checked_remove_dir_all(self, &relocation_target_in_parent).unwrap();
                                    } else {
                                        fs::remove_file(&relocation_target_in_parent).unwrap();
                                    }
                                }

                                fs::rename(dir_entry.path(), &relocation_target_in_parent).unwrap();
                            }
                        }
                    }

                    let _ = checked_remove_dir_all(self, &temp_dir);
                }

                let hyper_dir = HyperDir::read(&dir_converting);
                let mut video = Video::read_dir(self, &hyper_dir, playlist.copy_inherited_options(), &site_mut);

                {
                    let video_mut = Arc::<Video>::get_mut(&mut video).unwrap(); // always succeeds, single-owned at this point

                    // We only copy fields from the playlist to the new video,
                    // if the video was not initialized from a single video that
                    // already existed in the playlist.
                    if playlist.videos.is_empty() {
                        // TODO: We could consider copying a playlist banner to be the new video poster here
                        if let Some(description) = &playlist.description {
                            video_mut.description = Some(description.clone());
                        }
                        if !playlist.links.is_empty() {
                            video_mut.links = playlist.links.clone();
                        }
                        if let Some(title) = &playlist.title {
                            video_mut.title = Some(title.clone());
                        }
                    }

                    // If the playlist was offline, we ensure that the video
                    // that was indirectly offline as its child, stays offline
                    // when it replaces the playlist
                    if playlist.offline_local() && !video_mut.offline_local() {
                        video_mut.offline_local_set(true);
                    }

                    // If the playlist was unlisted, we ensure that the video
                    // that was indirectly unlisted as its child, stays unlisted
                    // when it replaces the playlist
                    if playlist.unlisted_local() && !video_mut.unlisted_local() {
                        video_mut.unlisted_local_set(true);
                    }
                }

                video.write_manifest(self);

                self.update_from_filesystem(&dir_converting, &site_mut);
            }
        }

        Ok(())
    }

    pub fn delete_collection(&self, site_path: &SitePath) -> bool {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Collection(collection) = &site_mut.content {
                collection.unpersist(self);
                site_mut.content = SiteContent::Empty;
                self.replace_site(site_mut);
                return true;
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if collection_mut.delete_collection(self, site_path) {
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
                return true;
            }
        }

        false
    }

    pub fn delete_playlist(&self, site_path: &SitePath) -> bool {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Playlist(playlist) = &site_mut.content {
                playlist.unpersist(self);
                site_mut.content = SiteContent::Empty;
                self.replace_site(site_mut);
                return true;
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if collection_mut.delete_playlist(self, site_path) {
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
                return true;
            }
        }

        false
    }

    /// Deletes the video at path (if found) and in case of success returns
    // its parent (collection or playlist or none if it was at the site root).
    pub fn delete_video(&self, site_path: &SitePath) -> Result<Option<VideoParent>, ()> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Video(video) = &site_mut.content {
                video.unpersist(self);
                site_mut.content = SiteContent::Empty;
                self.replace_site(site_mut);
                return Ok(None);
            }
        } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
            let mut playlist_mut = (**playlist).clone();

            if playlist_mut.delete_video(self, site_path) {
                let playlist_new = Arc::new(playlist_mut);
                *playlist = Arc::clone(&playlist_new);
                self.replace_site(site_mut);
                return Ok(Some(VideoParent::Playlist(playlist_new)));
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(video_parent) = collection_mut.delete_video(self, site_path) {
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
    pub fn get_any(&self, site_path: &SitePath) -> GetAnyResult {
        self.site.lock().unwrap().get_any(site_path)
    }

    /// Returns the collection at the given path.
    pub fn get_collection(&self, site_path: &SitePath) -> Option<Arc<Collection>> {
        self.site.lock().unwrap().get_collection(site_path)
    }

    /// Returns cloned `Language`, allowing subsequent read-only usage
    /// of the language without keeping a lock on it.
    pub fn get_editor_language(&self) -> Language {
        self.editor_language.lock().unwrap().clone()
    }

    /// Returns cloned `TreeViewOrder`, allowing subsequent read-only usage
    /// of the tree view order without keeping a lock on it.
    pub fn get_editor_tree_view_order(&self) -> TreeViewOrder {
        self.editor_tree_view_order.lock().unwrap().clone()
    }

    /// Returns cloned `Theme`, allowing subsequent read-only usage
    /// of the theme without keeping a lock on it.
    pub fn get_editor_theme(&self) -> Theme {
        self.editor_theme.lock().unwrap().clone()
    }

    /// Returns the playlist at the given path.
    pub fn get_playlist(&self, site_path: &SitePath) -> Option<Arc<Playlist>> {
        self.site.lock().unwrap().get_playlist(site_path)
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
    pub fn get_video(&self, site_path: &SitePath) -> Option<Arc<Video>> {
        self.site.lock().unwrap().get_video(site_path)
    }

    pub fn new(
        build_dir: PathBuf,
        cache: Mutex<Cache>,
        cache_dir: PathBuf,
        editor_language: &str,
        editor_theme: &str,
        editor_tree_view_order: TreeViewOrder,
        site: Arc<Site>,
        site_dir: PathBuf
    ) -> Context {
        Context {
            build_dir,
            cache,
            cache_dir,
            deploying: Mutex::new(DeploymentState::new()),
            editor_language: Mutex::new(Language::from_code(editor_language)),
            editor_theme: Mutex::new(Theme::from_key(editor_theme).unwrap_or_else(|_| Theme::default())),
            editor_tree_view_order: Mutex::new(editor_tree_view_order),
            job_queue: Mutex::new(Vec::new()),
            jobs_completed: Mutex::new(Vec::new()),
            processing: Mutex::new(Processing::new()),
            processing_errors: Mutex::new(Vec::new()),
            site: Mutex::new(site),
            site_dir
        }
    }

    fn queue_collection_jobs(&self, collection: &Arc<Collection>, site: &Arc<Site>, job_kinds: &mut Vec<JobKind>) {
        if collection.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            job_kinds.push(JobKind::CollectionFastTasks(Arc::clone(collection)));
        }

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

                job.assigned_at.is_none()
            })
            .reduce(|selected, iterated|
                match iterated.cmp_priority(selected) { 
                    Ordering::Greater => iterated,
                    _ => selected
                } 
            )
            .map(|job| {
                job.assigned_at = Some(SystemTime::now());
                job.clone()
            })
    }

    pub fn update_collection<T>(
        &self,
        mutation: impl FnOnce(&mut Collection) -> T,
        site_path: &SitePath
    ) -> CollectionUpdate<T> {
        let mut site_mut = self.get_site_mut();

        let mut result = CollectionUpdate::NotFound;

        if site_path.is_root_path() {
            if let SiteContent::Collection(collection) = &mut site_mut.content {
                let mut collection_mut = (**collection).clone();
                let optional_return_value = mutation(&mut collection_mut);
                *collection = Arc::new(collection_mut);

                result = CollectionUpdate::Updated {
                    collection_updated: Arc::clone(collection),
                    optional_return_value
                };
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            result = collection_mut.update_collection(mutation, site_path);

            if let CollectionUpdate::Updated { .. } = result {
                *collection = Arc::new(collection_mut);
            }
        }

        if let CollectionUpdate::Updated { .. } = result {
             self.replace_site(site_mut);
        }

        result
    }

    /// Based on the passed (absolute) path determines from which site path
    /// downwards (e.g. from the root, from a subcollection, etc.) we need to
    /// update our site tree (picking up what changed in the filesystem).
    /// When passed a file path (which is allowed, e.g. a manifest, video
    /// file, etc.) the update still happens at the directory level, i.e.
    /// from the site path that contains the respective file.
    pub fn update_from_filesystem(&self, path: &Path, site: &Site) {
        let dir_path = if path.is_dir() {
            path
        } else if path.is_file() {
            path.parent().unwrap()
        } else {
            return;
        };

        let site_path = SitePath::from_path(self, dir_path);

        if site_path.is_root_path() {
            self.replace_site(Site::read_dir(self, &self.site_dir));
        } else {
            let mut site_mut = self.get_site_mut();

            if let SiteContent::Collection(collection) = &mut site_mut.content {
                let mut collection_mut = (**collection).clone();
                collection_mut.update_descendant_from_filesystem(self, site, &site_path);
                *collection = Arc::new(collection_mut);
                self.replace_site(site_mut);
            } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
                let mut playlist_mut = (**playlist).clone();
                playlist_mut.update_descendant_from_filesystem(self, site, &site_path);
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

    pub fn update_playlist<T>(
        &self,
        mutation: impl FnOnce(&mut Playlist) -> T,
        site_path: &SitePath
    ) -> PlaylistUpdate<T> {
        let mut site_mut = self.get_site_mut();

        let mut result = PlaylistUpdate::NotFound;

        if site_path.is_root_path() {
            if let SiteContent::Playlist(playlist) = &mut site_mut.content {
                let mut playlist_mut = (**playlist).clone();
                let optional_return_value = mutation(&mut playlist_mut);
                *playlist = Arc::new(playlist_mut);

                result = PlaylistUpdate::Updated {
                    optional_return_value,
                    playlist_updated: Arc::clone(playlist)
                };
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            result = collection_mut.update_playlist(mutation, site_path);

            if let PlaylistUpdate::Updated { .. } = result {
                *collection = Arc::new(collection_mut);
            }
        }

        if let PlaylistUpdate::Updated { .. } = result {
             self.replace_site(site_mut);
        }

        result
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
        site_path: &SitePath
    ) -> Result<Arc<Video>, ()> {
        let mut site_mut = self.get_site_mut();

        let mut video_new = None;

        if site_path.is_root_path() {
            if let SiteContent::Video(video) = &mut site_mut.content {
                let mut video_mut = (**video).clone();
                mutation(&mut video_mut);
                *video = Arc::new(video_mut);
                video_new = Some(Arc::clone(video));
            }
        } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
            let mut playlist_mut = (**playlist).clone();

            if let Ok(video) = playlist_mut.update_video(mutation, site_path) {
                *playlist = Arc::new(playlist_mut);
                video_new = Some(video);
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(video) = collection_mut.update_video(mutation, site_path) {
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

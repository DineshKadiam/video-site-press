// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use indoc::formatdoc;
use slug::slugify;

use crate::{
    Cache,
    Collection,
    CompletedJob,
    DeploymentState,
    HyperDir,
    InheritedOptions,
    JobDesc,
    JobProgress,
    Language,
    Playlist,
    QueuedJob,
    Site,
    SiteContent,
    SitePath,
    Theme,
    TreeViewOrder,
    Video,
    Worker
};
use crate::util::{
    checked_remove_dir_all,
    nanoid,
    string_from_os_str
};

pub struct Context {
    pub build_dir: PathBuf,
    pub cache: Mutex<Cache>,
    pub cache_dir: PathBuf,
    pub completed_jobs: Mutex<Vec<CompletedJob>>,
    pub deploying: Mutex<DeploymentState>,
    pub editor_language: Mutex<Language>,
    pub editor_theme: Mutex<Theme>,
    pub editor_tree_view_order: Mutex<TreeViewOrder>,
    pub job_queue: Mutex<Vec<QueuedJob>>,
    pub site: Mutex<Arc<Site>>,
    pub site_dir: PathBuf,
}

pub enum GetAnyResult {
    Collection(Arc<Collection>),
    None,
    Playlist(Arc<Playlist>),
    Video(Arc<Video>)
}

/// An update to a collection happens by cloning the existing collection,
/// running a closure on it that mutates it into the new, updated state and
/// lastly replacing the existing collection in the site tree with our new,
/// updated version. The calling code then receives a reference to the new
/// collection, and if it returned anything in the closure that performed the
/// mutation, it is passed back that value through `optional_return_value`.
/// Any persistence on disk (manifest, assets) must be performed by the
/// caller after a successful update.
pub enum CollectionUpdate<T> {
    /// When either nothing, or something else (a playlist or video), exists
    /// at the provided site_path.
    NotFound,
    /// Updating is prohibited while there are pending (manifest) errors as
    /// this could potentially discard the entire manifest content due to
    /// just a simple simple syntax error.
    PendingErrors { collection: Arc<Collection> },
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
/// it is passed back that value through `optional_return_value`. Any
/// persistence on disk (manifest, assets) must be performed by the caller
/// after a successful update.
pub enum PlaylistUpdate<T> {
    /// When either nothing, or something else (a collection or video), exists
    /// at the provided site_path.
    NotFound,
    /// Updating is prohibited while there are pending (manifest) errors as
    /// this could potentially discard the entire manifest content due to
    /// just a simple simple syntax error.
    PendingErrors { playlist: Arc<Playlist> },
    Updated {
        optional_return_value: T,
        playlist_updated: Arc<Playlist>
    }
}

/// An update to the site happens by cloning the existing site, running a
/// closure on it that mutates it into a new, updated state and lastly
/// replacing the existing site with our new, updated version. Any
/// persistence on disk (manifest) must be performed by the caller after a
/// successful update.
pub enum SiteUpdate {
    /// Updating is prohibited while there are pending (manifest) errors as
    /// this could potentially discard the entire manifest content due to
    /// just a simple simple syntax error.
    PendingErrors { site: Arc<Site> },
    Success { site_updated: Arc<Site> }
}

/// An update to a video happens by cloning the existing video, running a
/// closure on it that mutates it into a new, updated state and lastly
/// replacing the existing video in the site tree with our new, updated
/// version. The closure either confirms success by returning Ok(T) in which
/// T is an optional return value or can signal failure by returning Err(String).
/// The calling code receives a reference to the new video, and if
/// it returned anything in the closure that performed the mutation, it is
/// passed back that value through `optional_return_value` (or the Error(String)
/// variant value in case of failure). Any persistence on disk
/// (manifest, assets) must be performed by the caller after a successful
/// update.
pub enum VideoUpdate<T> {
    Error(String),
    /// When either nothing, or something else (a collection or playlist),
    /// exists at the provided site_path.
    NotFound,
    /// Updating is prohibited while there are pending (manifest) errors as
    /// this could potentially discard the entire manifest content due to
    /// just a simple simple syntax error.
    PendingErrors { video: Arc<Video> },
    Success {
        optional_return_value: T,
        video_updated: Arc<Video>
    }
}

pub enum VideoParent {
    Collection(Arc<Collection>),
    Playlist(Arc<Playlist>)
}

impl Context {
    /// Blocks until all jobs in the job queue have been processed.
    /// This returns Err(…) if one or more jobs fail, but will still
    /// await all remaining jobs before doing so.
    pub fn await_job_queue(&self) -> Result<(), String> {
        let mut previous_failed_jobs = 0;
        let mut previous_pending_jobs = 0;

        loop {
            let (failed_jobs, pending_jobs) = {
                let job_queue = self.job_queue.lock().unwrap();

                let mut failed_jobs = 0;
                let mut pending_jobs = 0;

                for job in job_queue.iter() {
                    if job.failure.is_some() {
                        failed_jobs += 1;
                    } else {
                        pending_jobs += 1;
                    }
                }

                if pending_jobs == 0 {
                    break;
                }

                (failed_jobs, pending_jobs)
            };

            if failed_jobs != previous_failed_jobs || pending_jobs != previous_pending_jobs {
                if failed_jobs > 0 {
                    println!("{failed_jobs} jobs have failed, waiting for {pending_jobs} pending jobs");
                } else {
                    println!("Waiting for {pending_jobs} pending jobs");
                }

                previous_failed_jobs = failed_jobs;
                previous_pending_jobs = pending_jobs;
            }

            thread::sleep(Duration::from_secs(1));
        }


        if previous_failed_jobs == 0 {
            Ok(())
        } else {
            let job_queue = self.job_queue.lock().unwrap();

            let errors = job_queue
                .iter()
                .map(|job| {
                    let error = &job.failure.as_ref().unwrap().error;
                    format!("- {error}")
                })
                .collect::<Vec<String>>()
                .join("\n");

            let message = formatdoc!("
                One or more background jobs failed:
                {errors}
            ");

            Err(message)
        }
    }

    /// Moves a finished job from the job queue to the completed jobs list.
    pub fn complete_job(&self, id: &str) {
        let completed_job = {
            let mut job_queue = self.job_queue.lock().unwrap();

            job_queue
                .iter()
                .position(|job| job.id == id)
                .map(|index| {
                    let queued_job = job_queue.swap_remove(index);
                    queued_job.complete()
                })
                .unwrap()
        };

        self.completed_jobs.lock().unwrap().push(completed_job);
    }

    pub fn compute_job_queue(&self) {
        let site = self.get_site();

        let mut job_descs = Vec::new();

        match &site.content {
            SiteContent::Collection(collection) => self.queue_collection_jobs(collection, &mut job_descs, &site),
            SiteContent::Playlist(playlist) => self.queue_playlist_jobs(&mut job_descs, playlist, &site),
            SiteContent::Video(video) => self.queue_video_jobs(&mut job_descs, &site, video),
            _ => ()
        }

        if !job_descs.is_empty() {
            self.extend_job_queue(job_descs);
        }
    }

    /// Take the playlist or video at the given path (or if the site is still
    /// empty, the empty site itself) and convert it to a collection. This
    /// may fail if we're converting from a video that is inside a playlist,
    /// as a playlist can not contain a collection.
    pub fn convert_to_collection(&self, site_path: &SitePath) -> Result<Arc<Collection>, String> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            match &site_mut.content {
                SiteContent::Collection(collection) => Ok(Arc::clone(collection)),
                SiteContent::Empty => {
                    let hyper_dir = HyperDir::read(&self.site_dir);
                    let mut collection_arc = Collection::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                    {
                        // Apply the name of the site_dir as initial title
                        let collection_mut = Arc::<Collection>::get_mut(&mut collection_arc).unwrap(); // always succeeds, single-owned at this point
                        collection_mut.title = Some(string_from_os_str(self.site_dir.file_name().unwrap()));
                    }

                    collection_arc.write_manifest(self);
                    site_mut.content = SiteContent::Collection(Arc::clone(&collection_arc));
                    self.replace_site(site_mut);

                    Ok(collection_arc)
                }
                SiteContent::Invalid { error } => {
                    let message = format!("The site is currently in an invalid state that needs to be manually resolved first ({error})");
                    Err(message)
                }
                SiteContent::Playlist(playlist) => {
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
                    site_mut.content = SiteContent::Collection(Arc::clone(&collection));
                    self.replace_site(site_mut);

                    Ok(collection)
                }
                SiteContent::Video(video) => {
                    let slug = match &video.title {
                        Some(title) => slugify(title),
                        None => match video.highest_quality_video_file() {
                            Some(version) => {
                                slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
                            }
                            // TODO: Although unlikely, technically this could conflict, so we should probably
                            //       check for collision and if so de-duplicate this slug before we go ahead.
                            None => String::from("video")
                        }
                    };

                    let video_dir = self.site_dir.join(&slug);
                    fs::create_dir(&video_dir).unwrap();

                    // Move existing files from the site_dir to the newly created video_dir
                    if let Ok(dir_entries) = self.site_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let file_name = dir_entry.file_name();
                                let file_name_str = file_name.to_string_lossy();

                                // Skip site.eno, hidden files (e.g. cache/build dir) and the dir we copy to
                                if file_name_str.starts_with('.') ||
                                    file_name_str == "site.eno" ||
                                    file_name_str == slug {
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
                    site_mut.content = SiteContent::Collection(Arc::clone(&collection));
                    self.replace_site(site_mut);

                    Ok(collection)
                }
            }
        } else {
            match self.get_any(site_path) {
                GetAnyResult::Collection(collection) => Ok(collection),
                GetAnyResult::None => {
                    let language = self.get_editor_language();
                    Err(language.translations.the_resource_xxx_was_not_found_message(site_path.normalized()))
                }
                GetAnyResult::Playlist(playlist) => {
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

                    Ok(collection)
                }
                GetAnyResult::Video(video) => {
                    if self.get_any(&video.parent_site_path_unchecked()).is_playlist() {
                        return Err(String::from("A video inside a playlist can not be converted to a collection"));
                    }

                    let dir_converting = self.site_dir.join(video.site_path.filesystem_relative());

                    let slug = match &video.title {
                        Some(title) => slugify(title),
                        None => match video.highest_quality_video_file() {
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

                    Ok(collection)
                }
            }
        }
    }

    /// Take the collection or video at the given path (or if the site is
    /// still empty, the empty site itself) and convert it to a playlist.
    /// This may fail if we're converting from a collection and that
    /// collection happens to contain playlists or subcollections (which a
    /// playlist can not contain), or if we're converting from a video that
    /// is contained inside a playlist (as a playlist can not contain another
    /// playlist).
    pub fn convert_to_playlist(&self, site_path: &SitePath) -> Result<Arc<Playlist>, String> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            match &site_mut.content {
                SiteContent::Collection(collection) => {
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
                    site_mut.content = SiteContent::Playlist(Arc::clone(&playlist));
                    self.replace_site(site_mut);

                    Ok(playlist)
                }
                SiteContent::Empty => {
                    let hyper_dir = HyperDir::read(&self.site_dir);
                    let mut playlist_arc = Playlist::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                    {
                        // Apply the name of the site_dir as initial title
                        let playlist_mut = Arc::<Playlist>::get_mut(&mut playlist_arc).unwrap(); // always succeeds, single-owned at this point
                        playlist_mut.title = Some(string_from_os_str(self.site_dir.file_name().unwrap()));
                    }

                    playlist_arc.write_manifest(self);
                    site_mut.content = SiteContent::Playlist(Arc::clone(&playlist_arc));
                    self.replace_site(site_mut);

                    Ok(playlist_arc)
                }
                SiteContent::Invalid { error } => {
                    let message = format!("The site is currently in an invalid state that needs to be manually resolved first ({error})");
                    Err(message)
                }
                SiteContent::Playlist(playlist) => Ok(Arc::clone(playlist)),
                SiteContent::Video(video) => {
                    let slug = match &video.title {
                        Some(title) => slugify(title),
                        None => match video.highest_quality_video_file() {
                            Some(version) => {
                                slugify(Path::new(&version.file_name).file_stem().unwrap().to_string_lossy())
                            }
                            // TODO: Although unlikely, technically this could conflict, so we should probably
                            //       check for collision and if so de-duplicate this slug before we go ahead.
                            None => String::from("video")
                        }
                    };

                    let video_dir = self.site_dir.join(&slug);
                    fs::create_dir(&video_dir).unwrap();

                    // Move existing files from the site_dir to the newly created video_dir
                    if let Ok(dir_entries) = self.site_dir.read_dir() {
                        for dir_entry_result in dir_entries {
                            if let Ok(dir_entry) = dir_entry_result {
                                let file_name = dir_entry.file_name();
                                let file_name_str = file_name.to_string_lossy();

                                // Skip site.eno, hidden files (e.g. cache/build dir) and the dir we copy to
                                if file_name_str.starts_with('.') ||
                                    file_name_str == "site.eno" ||
                                    file_name_str == slug {
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
                    site_mut.content = SiteContent::Playlist(Arc::clone(&playlist));
                    self.replace_site(site_mut);

                    Ok(playlist)
                }
            }
        } else {
            match self.get_any(site_path) {
                GetAnyResult::Collection(collection) => {
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

                    Ok(playlist)
                }
                GetAnyResult::None => {
                    let language = self.get_editor_language();
                    Err(language.translations.the_resource_xxx_was_not_found_message(site_path.normalized()))
                }
                GetAnyResult::Playlist(playlist) => Ok(playlist),
                GetAnyResult::Video(video) => {
                    if self.get_any(&video.parent_site_path_unchecked()).is_playlist() {
                        return Err("A video inside a playlist can not be converted to a playlist".to_string());
                    }

                    let dir_converting = self.site_dir.join(video.site_path.filesystem_relative());

                    let slug = match &video.title {
                        Some(title) => slugify(title),
                        None => match video.highest_quality_video_file() {
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

                    Ok(playlist)
                }
            }
        }
    }

    /// Take the collection or playlist at the given path (or if the site is still empty,
    /// the empty site itself) and convert it to a video. This only succeeds if we're
    /// converting from a collection or playlist that contains exactly one video (or none).
    pub fn convert_to_video(&self, site_path: &SitePath) -> Result<Arc<Video>, String> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            match &site_mut.content {
                SiteContent::Collection(collection) => {
                    if !collection.playlists.is_empty() {
                        return Err("A collection containing (a) playlist(s) can not be converted to a single video".to_string());
                    } else if !collection.subcollections.is_empty() {
                        return Err("A collection containing (a) subcollection(s) can not be converted to a single video".to_string());
                    } else if collection.videos.len() > 2 {
                        return Err("A collection containing two or more videos can not be converted to a single video".to_string());
                    }

                    let manifest_path = self.site_dir.join("collection.eno");
                    let _ = fs::remove_file(&manifest_path);

                    // If a (single) video dir exists in the collection, move
                    // every file/dir in it into the root, then delete it.
                    if let Some(video) = collection.videos.first() {
                        let temp_dir = self.site_dir.join(nanoid());

                        // Rename the video directory to unique temporary name, so we can copy
                        // files/folders within without any possible name collisions with the
                        // video directory name itself.
                        fs::rename(
                            self.site_dir.join(video.site_path.filesystem_relative()),
                            &temp_dir
                        ).unwrap();

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

                    site_mut.content = SiteContent::Video(Arc::clone(&video));
                    self.replace_site(site_mut);

                    Ok(video)
                }
                SiteContent::Empty => {
                    let hyper_dir = HyperDir::read(&self.site_dir);
                    let mut video_arc = Video::read_dir(self, &hyper_dir, InheritedOptions::default(), &site_mut);

                    {
                        // Apply the name of the site_dir as initial title
                        let video_mut = Arc::<Video>::get_mut(&mut video_arc).unwrap(); // always succeeds, single-owned at this point
                        video_mut.title = Some(string_from_os_str(self.site_dir.file_name().unwrap()));
                    }

                    video_arc.write_manifest(self);
                    site_mut.content = SiteContent::Video(Arc::clone(&video_arc));
                    self.replace_site(site_mut);

                    Ok(video_arc)
                }
                SiteContent::Invalid { error } => {
                    let message = format!("The site is currently in an invalid state that needs to be manually resolved first ({error})");
                    Err(message)
                }
                SiteContent::Playlist(playlist) => {
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

                    site_mut.content = SiteContent::Video(Arc::clone(&video));
                    self.replace_site(site_mut);

                    Ok(video)
                }
                SiteContent::Video(video) => Ok(Arc::clone(video))
            }
        } else {
            match self.get_any(site_path) {
                GetAnyResult::Collection(collection) => {
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

                    Ok(video)
                }
                GetAnyResult::None => {
                    let language = self.get_editor_language();
                    Err(language.translations.the_resource_xxx_was_not_found_message(site_path.normalized()))
                }
                GetAnyResult::Playlist(playlist) => {
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

                    Ok(video)
                }
                GetAnyResult::Video(video) => Ok(video)
            }
        }
    }

    /// If succeeding, the resulting Ok() contains the parent of the deleted
    /// collection (if there is one).
    pub fn delete_collection(&self, site_path: &SitePath) -> Result<Option<Arc<Collection>>, ()> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Collection(collection) = &site_mut.content {
                collection.unpersist(self);
                site_mut.content = SiteContent::Empty;
                self.replace_site(site_mut);

                return Ok(None);
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(deeper_parent) = collection_mut.delete_collection(self, site_path) {
                *collection = Arc::new(collection_mut);
                let parent = deeper_parent.or(Some(Arc::clone(collection)));
                self.replace_site(site_mut);

                return Ok(parent);
            }
        }

        Err(())
    }

    /// If succeeding, the resulting Ok() contains the parent of the deleted
    /// playlist (if there is one).
    pub fn delete_playlist(&self, site_path: &SitePath) -> Result<Option<Arc<Collection>>, ()> {
        let mut site_mut = self.get_site_mut();

        if site_path.is_root_path() {
            if let SiteContent::Playlist(playlist) = &site_mut.content {
                playlist.unpersist(self);
                site_mut.content = SiteContent::Empty;
                self.replace_site(site_mut);

                return Ok(None);
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(deeper_parent) = collection_mut.delete_playlist(self, site_path) {
                *collection = Arc::new(collection_mut);
                let parent = deeper_parent.or(Some(Arc::clone(collection)));
                self.replace_site(site_mut);

                return Ok(parent);
            }
        }

        Err(())
    }

    /// If succeeding, the resulting Ok() contains the parent of the deleted
    /// video (if there is one).
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
                *playlist = Arc::new(playlist_mut);
                let parent = Some(VideoParent::Playlist(Arc::clone(playlist)));
                self.replace_site(site_mut);

                return Ok(parent);
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            if let Ok(deeper_parent) = collection_mut.delete_video(self, site_path) {
                *collection = Arc::new(collection_mut);
                let parent = deeper_parent.or(Some(VideoParent::Collection(Arc::clone(collection))));
                self.replace_site(site_mut);

                return Ok(parent);
            }
        }

        Err(())
    }

    /// Takes the given job descriptions, and creates jobs for each of them that is not
    /// already being worked on anyway.
    pub fn extend_job_queue(&self, job_descs: Vec<JobDesc>) {
        if let Ok(mut job_queue) = self.job_queue.lock() {
            for job_desc in job_descs {
                if !job_queue
                    .iter()
                    .any(|existing_job| existing_job.desc == job_desc) {
                    job_queue.push(QueuedJob::new(job_desc));
                }
            }
        }
    }

    pub fn fail_job(&self, error: String, id: &str) {
        let mut job_queue = self.job_queue.lock().unwrap();

        if let Some(job) = job_queue
            .iter_mut()
            .find(|job| job.id == id) {
            job.fail(error);
        }
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
            completed_jobs: Mutex::new(Vec::new()),
            deploying: Mutex::new(DeploymentState::new()),
            editor_language: Mutex::new(Language::from_code(editor_language)),
            editor_theme: Mutex::new(Theme::from_key(editor_theme).unwrap_or_else(|_| Theme::default())),
            editor_tree_view_order: Mutex::new(editor_tree_view_order),
            job_queue: Mutex::new(Vec::new()),
            site: Mutex::new(site),
            site_dir
        }
    }

    fn queue_collection_jobs(
        &self,
        collection: &Arc<Collection>,
        job_descs: &mut Vec<JobDesc>,
        site: &Arc<Site>
    ) {
        if collection.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            let collection_fast_tasks = JobDesc::CollectionFastTasks {
                collection: Arc::clone(collection)
            };

            job_descs.push(collection_fast_tasks);
        }

        for subcollection in &collection.subcollections {
            self.queue_collection_jobs(subcollection, job_descs, site);
        }
        for playlist in &collection.playlists {
            self.queue_playlist_jobs(job_descs, playlist, site);
        }
        for video in &collection.videos {
            self.queue_video_jobs(job_descs, site, video);
        }
    }

    pub fn queue_job(&self, job_desc: JobDesc) {
        if let Ok(mut job_queue) = self.job_queue.lock() {
            if !job_queue
                .iter()
                .any(|existing_job| existing_job.desc == job_desc) {
                job_queue.push(QueuedJob::new(job_desc));
            }
        }
    }

    fn queue_playlist_jobs(
        &self,
        job_descs: &mut Vec<JobDesc>,
        playlist: &Arc<Playlist>,
        site: &Arc<Site>
    ) {
        if playlist.banner.as_ref().is_some_and(|banner| banner.assets.is_none()) {
            let playlist_fast_tasks = JobDesc::PlaylistFastTasks {
                playlist: Arc::clone(playlist)
            };

            job_descs.push(playlist_fast_tasks);
        }

        for video in &playlist.videos {
            self.queue_video_jobs(job_descs, site, video);
        }
    }

    /// Determine what needs to be computed for this video and append the
    /// required jobs to the passed queue.
    fn queue_video_jobs(
        &self,
        job_descs: &mut Vec<JobDesc>,
        site: &Arc<Site>,
        video: &Arc<Video>
    ) {
        if video.audio_files.iter().any(|audio_file| audio_file.audio_meta.is_none()) {
            let audio_meta_job_desc = JobDesc::AudioMeta {
                video: Arc::clone(video)
            };

            job_descs.push(audio_meta_job_desc);
        }

        if video.video_files.iter().any(|video_file| video_file.video_meta.is_none()) ||
           (video.poster.is_none() && !video.video_files.is_empty()) ||
           video.poster.as_ref().is_some_and(|poster|
                poster.assets.is_none() ||
                poster.assets.as_ref().is_some_and(|assets|
                    assets.poster_aspect != site.poster_aspect.numeric()
                ) ||
                poster.poster_meta.is_none()
            ) {
            let video_fast_tasks_job_desc = JobDesc::VideoFastTasks {
                poster_aspect: site.poster_aspect.numeric(),
                video: Arc::clone(video)
            };

            job_descs.push(video_fast_tasks_job_desc);
        }
    }

    /// Replace the immutable `Arc<Site>` at the site tree root,
    /// updating the site tree to a new version.
    pub fn replace_site(&self, site_new: Site) {
        *self.site.lock().unwrap() = Arc::new(site_new);
    }

    /// Take the job with the highest priority that isn't assigned yet.
    /// Note that the job stays in the queue, just marked as assigned.
    pub fn take_next_job(&self, worker: &Worker) -> Option<QueuedJob> {
        let mut job_queue = self.job_queue.lock().unwrap();

        job_queue
            .iter_mut()
            .filter(|queued_job| {
                queued_job.active_assignment.is_none() &&
                queued_job.failure.is_none() &&
                !(worker.high_priority_only && !queued_job.desc.is_high_priority())
            })
            .reduce(|selected, iterated|
                match iterated.cmp_priority(selected) {
                    Ordering::Greater => iterated,
                    _ => selected
                }
            )
            .map(|queued_job| {
                queued_job.assign(worker.number);
                queued_job.clone()
            })
    }

    pub fn update_collection<T>(
        &self,
        mutation: impl FnOnce(&mut Collection) -> T,
        site_path: &SitePath
    ) -> CollectionUpdate<T> {
        let mut site_mut = self.get_site_mut();

        let result = if site_path.is_root_path() {
            if let SiteContent::Collection(collection) = &mut site_mut.content {
                if collection.manifest_errors.is_empty() {
                    let mut collection_mut = (**collection).clone();
                    let optional_return_value = mutation(&mut collection_mut);
                    *collection = Arc::new(collection_mut);

                    CollectionUpdate::Updated {
                        collection_updated: Arc::clone(collection),
                        optional_return_value
                    }
                } else {
                    CollectionUpdate::PendingErrors {
                        collection: Arc::clone(collection)
                    }
                }
            } else {
                CollectionUpdate::NotFound
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            let deeper_result = collection_mut.update_subcollection(mutation, site_path);

            if let CollectionUpdate::Updated { .. } = deeper_result {
                *collection = Arc::new(collection_mut);
            }

            deeper_result
        } else {
            CollectionUpdate::NotFound
        };

        if let CollectionUpdate::Updated { .. } = result {
             self.replace_site(site_mut);
        }

        result
    }

    // TODO: This functions immutably borrows site, but then obtains its own
    //       site_mut handle through get_site_mut() which could also be used.
    //       Refactoring this however is probably not so easy because it looks
    //       like we will run into parallel mutable/immutable borrow issues.
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

    /// Update progress info on the active job with the given id
    pub fn update_job_progress(&self, id: &str, progress: JobProgress) {
        let mut job_queue = self.job_queue.lock().unwrap();

        if let Some(job) = job_queue
            .iter_mut()
            .find(|job| job.id == id) {
            let assignment = job.active_assignment.as_mut().unwrap();
            assignment.last_progress = Some(progress);
        }
    }

    pub fn update_playlist<T>(
        &self,
        mutation: impl FnOnce(&mut Playlist) -> T,
        site_path: &SitePath
    ) -> PlaylistUpdate<T> {
        let mut site_mut = self.get_site_mut();

        let result = if site_path.is_root_path() {
            if let SiteContent::Playlist(playlist) = &mut site_mut.content {
                if playlist.manifest_errors.is_empty() {
                    let mut playlist_mut = (**playlist).clone();
                    let optional_return_value = mutation(&mut playlist_mut);
                    *playlist = Arc::new(playlist_mut);

                    PlaylistUpdate::Updated {
                        optional_return_value,
                        playlist_updated: Arc::clone(playlist)
                    }
                } else {
                    PlaylistUpdate::PendingErrors {
                        playlist: Arc::clone(playlist)
                    }
                }
            } else {
                PlaylistUpdate::NotFound
            }
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            let deeper_result = collection_mut.update_playlist(mutation, site_path);

            if let PlaylistUpdate::Updated { .. } = deeper_result {
                *collection = Arc::new(collection_mut);
            }

            deeper_result
        } else {
            PlaylistUpdate::NotFound
        };

        if let PlaylistUpdate::Updated { .. } = result {
             self.replace_site(site_mut);
        }

        result
    }

    pub fn update_site(
        &self,
        mutation: impl FnOnce(&mut Site),
    ) -> SiteUpdate {
        let site = self.get_site();

        if site.manifest_errors.is_empty() {
            let mut site_mut = (*site).clone();

            mutation(&mut site_mut);

            let site_updated = Arc::new(site_mut);

            *self.site.lock().unwrap() = Arc::clone(&site_updated);

            SiteUpdate::Success { site_updated }
        } else {
            SiteUpdate::PendingErrors { site }
        }
    }

    pub fn update_video<T>(
        &self,
        mutation: impl FnOnce(&mut Video) -> Result<T, String>,
        site_path: &SitePath
    ) -> VideoUpdate<T> {
        let mut site_mut = self.get_site_mut();

        let result = if site_path.is_root_path() {
            if let SiteContent::Video(video) = &mut site_mut.content {
                if video.manifest_errors.is_empty() {
                    let mut video_mut = (**video).clone();

                    match mutation(&mut video_mut) {
                        Ok(optional_return_value) => {
                            *video = Arc::new(video_mut);

                            VideoUpdate::Success {
                                optional_return_value,
                                video_updated: Arc::clone(video)
                            }
                        }
                        Err(err) => VideoUpdate::Error(err)
                    }
                } else {
                    VideoUpdate::PendingErrors {
                        video: Arc::clone(video)
                    }
                }
            } else {
                VideoUpdate::NotFound
            }
        } else if let SiteContent::Playlist(playlist) = &mut site_mut.content {
            let mut playlist_mut = (**playlist).clone();

            let deeper_result = playlist_mut.update_video(mutation, site_path);

            if let VideoUpdate::Success { .. } = deeper_result {
                *playlist = Arc::new(playlist_mut);
            }

            deeper_result
        } else if let SiteContent::Collection(collection) = &mut site_mut.content {
            let mut collection_mut = (**collection).clone();

            let deeper_result = collection_mut.update_video(mutation, site_path);

            if let VideoUpdate::Success { .. } = deeper_result {
                *collection = Arc::new(collection_mut);
            }

            deeper_result
        } else {
            VideoUpdate::NotFound
        };

        if let VideoUpdate::Success { .. } = result {
            self.replace_site(site_mut);
        }

        result
    }
}

impl GetAnyResult {
    pub fn is_playlist(&self) -> bool {
        match self {
            GetAnyResult::Collection(_) => false,
            GetAnyResult::None => false,
            GetAnyResult::Playlist(_) => true,
            GetAnyResult::Video(_) => false
        }
    }
}

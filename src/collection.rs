// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::{self, Ordering};
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::str::Split;
use std::sync::Arc;

use chrono::NaiveDate;

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
    TreeViewOrder,
    Video,
    VideoOrder,
    VideoParent
};
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::checked_remove_dir_all;

const COLLECTION_OPTIONS: &[&str] = &[
    "banner",
    "description",
    "link",
    "offline",
    "title",
    "unlisted",
    "video_order"
];

#[derive(Clone, Debug)]
pub struct Collection {
    pub banner: Option<Banner>,
    pub description: Option<String>,
    pub errors: Vec<String>,
    pub links: Vec<Link>,
    pub offline: bool,
    pub path: String,
    pub playlists: Vec<Arc<Playlist>>,
    pub subcollections: Vec<Arc<Collection>>,
    pub title: Option<String>,
    pub unlisted: bool,
    pub video_order: VideoOrder,
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


        for element in document.elements() {
            match element.key() {
                "banner" => 'banner: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(attributes) = field.attributes() {
                            let mut description = None;
                            let mut file = None;

                            for attribute in attributes {
                                match attribute.key() {
                                    "description" => {
                                        if let Some(value) = attribute.value() {
                                            description = Some(value.to_string());
                                        }
                                    }
                                    "file" => {
                                        if let Some(value) = attribute.value() {
                                            let absolute_path = manifest_path.parent().unwrap().join(value);

                                            if absolute_path.exists() {
                                                file = Some((absolute_path, value.to_string()));
                                            } else {
                                                let message = format!("The referenced file was not found ({})", absolute_path.display());
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.errors.push(error);
                                            }
                                        }
                                    }
                                    _ => {
                                        let message = "The key/name of this attribute was not recognized, only 'description' and 'file' are recognized inside an banner field";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            if let Some((absolute_path, file_name)) = file {
                                let file_meta = FileMeta::new(&absolute_path);

                                let assets = context.cache.lock().unwrap().fetch_poster_assets(
                                    &file_meta,
                                    &file_name,
                                    &self.path,
                                    0.0 // TODO: Banners have no poster_aspect, pending better modeling we just hard-code it to 0 always
                                );

                                let banner = Banner::new(assets, description, file_meta, file_name);
                                self.banner = Some(banner);
                            }

                            break 'banner;
                        }
                    }

                    let message = "banner needs to be provided as a field with attributes, e.g.:\n\nbanner:\ndescription = Strange geometric patterns\nfile = patterns.jpg";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "description" => {
                    if let Ok(embed) = element.as_embed() {
                        if let Some(value) = embed.value() {
                            self.description = Some(value.to_string());
                        } else {
                            self.description = None;
                        }
                    } else {
                        let message = "The 'description' option needs to be provided as an embed, e.g.:\n-- description\nA description text\n--description";
                        let error = element_error(element, manifest_path, message);
                        self.errors.push(error);
                    }
                }
                "link" => 'link: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match HyperUrl::parse(value) {
                                    Ok(url) => {
                                        let link = Link::new(false, None, false, url);
                                        self.links.push(link);
                                    }
                                    Err(err) => {
                                        let message = format!("The url supplied for the link seems to be malformed ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'link;
                        } else if let Ok(attributes) = field.attributes() {
                            let mut hidden = false;
                            let mut label = None;
                            let mut rel_me = false;
                            let mut url = None;

                            for attribute in attributes {
                                match attribute.key() {
                                    "label" => {
                                        if let Some(value) = attribute.value() {
                                            label = Some(value.to_string());
                                        }
                                    }
                                    "url" => {
                                        if let Some(value) = attribute.value() {
                                            match HyperUrl::parse(value) {
                                                Ok(parsed_url) => url = Some(parsed_url),
                                                Err(err) => {
                                                    let message = format!("The url supplied for the link seems to be malformed ({err})");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    "verification" => {
                                        if let Some(value) = attribute.value() {
                                            match value {
                                                "rel-me" => {
                                                    hidden = false;
                                                    rel_me = true;
                                                }
                                                "rel-me-hidden" => {
                                                    hidden = true;
                                                    rel_me = true;
                                                }
                                                _ => {
                                                    let message = format!("The verification attribute value '{value}' is not recognized, allowed are 'rel-me' and 'rel-me-hidden'");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    other => {
                                        let message = format!("The attribute '{other}' is not recognized here (supported attributes are 'label', 'url' and 'verification'");
                                        let error = attribute_error(attribute, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            if let Some(url) = url {
                                let link = Link::new(hidden, label, rel_me, url);
                                self.links.push(link);
                            } else {
                                let message = "The link option must supply an url attribute at least, e.g.:\n\nlink:\nurl = https://example.com";
                                let error = element_error(element, manifest_path, message);
                                self.errors.push(error);
                            }

                            break 'link;
                        }
                    }

                    let message = "link must be provided as a basic field with a value (e.g. 'link: https://example.com') or in its extended form as a field with attributes, e.g.:\n\nlink:\nurl = https://example.com\nlabel = Example";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "offline" => {
                    if element.is_flag() {
                        self.offline = true;
                    } else {
                        let message = "offline needs to be provided as a flag, that is, exactly as 'offline' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.errors.push(error);
                    }
                }
                "title" => 'title: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                self.title = Some(value.to_string());
                            }

                            break 'title;
                        }
                    }

                    let message = "title needs to be provided as a field with a value, e.g.: 'title: The 90ies clip collection'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "unlisted" => {
                    if element.is_flag() {
                        self.unlisted = true;
                    } else {
                        let message = "unlisted needs to be provided as a flag, that is, exactly as 'unlisted' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.errors.push(error);
                    }
                }
                "video_order" => 'video_order: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match VideoOrder::from_key(value) {
                                    Ok(order) => {
                                        self.video_order = order;
                                    }
                                    Err(err) => {
                                        let message = format!("The supplied order was not recognized ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'video_order;
                        }
                    }

                    let message = "video_order needs to be provided as a field with a value, e.g.: 'video_order: release_date_asc'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                other => {
                    let message = not_supported_error("collection.eno", other, &[COLLECTION_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.errors.push(error);
                }
            }
        }
    }

    pub fn cmp_by_latest_release_date_desc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        b.latest_release_date().cmp(&a.latest_release_date())
    }

    // Collections without title are sorted to the front
    pub fn cmp_by_slug_then_title_asc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    }

    // Collections without title are sorted to the front
    pub fn cmp_by_slug_then_title_desc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => b_title.cmp(&a_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => b.slug().cmp(&a.slug())
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

                if subcollection_mut.delete_collection(context, path_parts) {
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
                        Some(cmp::max(latest_release_date, release_date))
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
                        Some(cmp::max(latest_release_date, release_date))
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
                        Some(cmp::max(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        *[latest_in_playlists, latest_in_subcollections, latest_in_videos]
            .iter()
            .max()
            .unwrap()
    }

    pub fn new(path: String) -> Collection {
        Collection {
            banner: None,
            description: None,
            errors: Vec::new(),
            links: Vec::new(),
            offline: false,
            path,
            playlists: Vec::new(),
            subcollections: Vec::new(),
            title: None,
            unlisted: false,
            video_order: VideoOrder::Default,
            videos: Vec::new()
        }
    }

    pub fn parent_path(&self) -> &Path {
        if self.path.is_empty() {
            Path::new("")
        } else {
            Path::new(&self.path).parent().unwrap()
        }
    }

    pub fn playlists_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Playlist>> {
        let mut playlists: Vec<Arc<Playlist>> = self.playlists.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => playlists.sort_unstable_by(Playlist::cmp_by_latest_release_date_desc),
            TreeViewOrder::TitleAsc => playlists.sort_unstable_by(Playlist::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => playlists.sort_unstable_by(Playlist::cmp_by_slug_then_title_desc)
        }

        playlists
    }

    pub fn public(&self) -> bool {
        !self.offline && !self.unlisted
    }

    pub fn public_playlists(&self) -> impl Iterator<Item = &Arc<Playlist>> {
        self.playlists.iter().filter(|playlist| playlist.public())
    }

    pub fn public_playlists_desc_by_release_date(&self) -> Vec<Arc<Playlist>> {
        let mut playlists: Vec<Arc<Playlist>> = self.public_playlists().cloned().collect();
        playlists.sort_unstable_by(Playlist::cmp_by_latest_release_date_desc);
        playlists
    }

    pub fn public_subcollections(&self) -> impl Iterator<Item = &Arc<Collection>> {
        self.subcollections.iter().filter(|subcollection| subcollection.public())
    }

    pub fn public_subcollections_desc_by_release_date(&self) -> Vec<Arc<Collection>> {
        let mut subcollections: Vec<Arc<Collection>> = self.public_subcollections().cloned().collect();
        subcollections.sort_unstable_by(Collection::cmp_by_latest_release_date_desc);
        subcollections
    }

    pub fn public_videos(&self) -> impl Iterator<Item = &Arc<Video>> {
        self.videos.iter().filter(|video| video.public())
    }

    pub fn public_videos_desc_by_release_date(&self) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.public_videos().cloned().collect();
        videos.sort_unstable_by(Video::cmp_by_release_date_desc);
        videos
    }

    pub fn public_videos_in_collection_video_order(&self) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.public_videos().cloned().collect();
        videos.sort_unstable_by(Video::cmp_by_video_order(&self.video_order));
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

    pub fn subcollections_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Collection>> {
        let mut subcollections: Vec<Arc<Collection>> = self.subcollections.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => subcollections.sort_unstable_by(Collection::cmp_by_latest_release_date_desc),
            TreeViewOrder::TitleAsc => subcollections.sort_unstable_by(Collection::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => subcollections.sort_unstable_by(Collection::cmp_by_slug_then_title_desc)
        }

        subcollections
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

    pub fn videos_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.videos.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => videos.sort_unstable_by(Video::cmp_by_video_order(&self.video_order)),
            TreeViewOrder::TitleAsc => videos.sort_unstable_by(Video::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => videos.sort_unstable_by(Video::cmp_by_slug_then_title_desc)
        }

        videos
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

        if self.offline {
            eno.push_str("offline\n");
        }

        if let Some(title) = &self.title {
            eno.push_str(&format!("title: {}\n", title));
        }

        if self.unlisted {
            eno.push_str("unlisted\n");
        }

        if self.video_order != VideoOrder::Default {
            eno.push_str(&format!("video_order: {}\n", self.video_order.key()));
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

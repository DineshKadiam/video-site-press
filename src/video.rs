// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use enolib::{Element, SectionElement};
use serde_derive::{Deserialize, Serialize};

use crate::{
    CacheItem,
    Collection,
    Context,
    FileMeta,
    HyperDir,
    HyperUrl,
    Link,
    Playlist,
    PosterFile,
    Site,
    SubtitleFile,
    Translations,
    VideoMetaCached
};
use crate::util::{checked_remove_dir_all, hash};

#[derive(Clone, Debug)]
pub struct Video {
    pub description: Option<String>,
    pub errors: Vec<String>,
    pub links: Vec<Link>,
    /// Relative to site dir, always without leading/trailing slashes
    pub path: String,
    pub poster: Option<PosterFile>,
    pub release_date: Option<NaiveDate>,
    pub sort_number: Option<i32>,
    pub subtitles: Vec<SubtitleFile>,
    pub title: Option<String>,
    /// Unlisted videos are not linked from the site anywhere,
    /// one needs to know the link to access them.
    pub unlisted: bool,
    // TODO: We cannot actually represent an audio only version here?
    //       (or only in an odd way - height/width are e.g. always present on VideoFile)
    pub versions: Vec<VideoFile>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoFile {
    pub file_meta: FileMeta,
    pub file_name: String,
    pub video_meta: Option<VideoMeta>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoMeta {
    pub audio_codec: Option<String>,
    /// Duration in seconds
    pub duration: f32,
    pub height: usize,
    pub video_codec: Option<String>,
    pub video_frame_rate: f32,
    pub width: usize
}

pub enum VideoParent {
    Collection(Arc<Collection>),
    Playlist(Arc<Playlist>)
}

impl Video {
    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path, site: &Site) {
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

        match document.optional_field("poster") {
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
                                        site.poster_aspect.numeric()
                                    );

                                    let poster = PosterFile::new(assets, description, file_meta, file_name);
                                    self.poster = Some(poster);
                                } else {
                                    self.errors.push(format!("Error in {}:{} (Path to poster is invalid, file not found)", manifest_path.display(), attribute.line_number()));
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

        match document.optional_field("sort_number") {
            Ok(Some(field)) => {
                match field.required_value::<String>() {
                    Ok(value) => match value.parse::<i32>() {
                        Ok(number) => {
                            self.sort_number = Some(number);
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

    // Videos without an earliest modified date date are sorted to the back
    pub fn cmp_by_earliest_modified_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        let a_earliest_modified = a.versions.iter().map(|video_file| video_file.file_meta.modified).min();
        let b_earliest_modified = b.versions.iter().map(|video_file| video_file.file_meta.modified).min();

        match (a_earliest_modified, b_earliest_modified) {
            (Some(a_modified), Some(b_modified)) => a_modified.cmp(&b_modified),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without an earliest modified date date are sorted to the back
    pub fn cmp_by_earliest_modified_desc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        let a_earliest_modified = a.versions.iter().map(|video_file| video_file.file_meta.modified).min();
        let b_earliest_modified = b.versions.iter().map(|video_file| video_file.file_meta.modified).min();

        match (a_earliest_modified, b_earliest_modified) {
            (Some(a_modified), Some(b_modified)) => b_modified.cmp(&a_modified),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without release date are sorted to the back
    pub fn cmp_by_release_date_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.release_date, &b.release_date) {
            (Some(a_release_date), Some(b_release_date)) => {
                match a_release_date.cmp(&b_release_date) {
                    Ordering::Equal => Video::cmp_by_earliest_modified_asc(a, b),
                    ordering => ordering
                }
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without release date are sorted to the back
    pub fn cmp_by_release_date_desc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.release_date, &b.release_date) {
            (Some(a_release_date), Some(b_release_date)) => {
                match b_release_date.cmp(&a_release_date) {
                    Ordering::Equal => Video::cmp_by_earliest_modified_desc(a, b),
                    ordering => ordering
                }
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without sort number are sorted to the back
    pub fn cmp_by_sort_number_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(&b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without sort number are sorted to the back
    pub fn cmp_by_sort_number_desc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => b_sort_number.cmp(&a_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without sort number and without release date are sorted to the back
    pub fn cmp_by_sort_number_then_release_date_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(&b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Video::cmp_by_release_date_asc(a, b)
        }
    }

    // Videos with title and videos with only a slug are sorted together
    pub fn cmp_by_title_or_slug_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        let a_title = a.title.as_deref().unwrap_or_else(|| a.slug());
        let b_title = b.title.as_deref().unwrap_or_else(|| b.slug());

        a_title.cmp(b_title)
    }

    // Videos with title and videos with only a slug are sorted together
    pub fn cmp_by_title_or_slug_desc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        let a_title = a.title.as_deref().unwrap_or_else(|| a.slug());
        let b_title = b.title.as_deref().unwrap_or_else(|| b.slug());

        b_title.cmp(a_title)
    }

    // Videos without title are sorted to the front
    pub fn cmp_by_slug_then_title_asc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(&b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    }

    // Videos without title are sorted to the front
    pub fn cmp_by_slug_then_title_desc(a: &Arc<Video>, b: &Arc<Video>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => b_title.cmp(&a_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => b.slug().cmp(&a.slug())
        }
    }

    pub fn highest_quality_version(&self) -> Option<&VideoFile> {
        let mut version_candidate: Option<(usize, &VideoFile)> = None;

        for version in &self.versions {
            if let Some(video_meta) = &version.video_meta {
                let pixels = video_meta.width * video_meta.height;
                if version_candidate.is_none() ||
                    version_candidate.is_some_and(|pixels_version|
                        pixels > pixels_version.0 ||
                        (pixels == pixels_version.0 && version.file_meta.size > pixels_version.1.file_meta.size)) {
                    version_candidate = Some((pixels, version));
                }
            }
        }

        version_candidate.map(|pixels_version| pixels_version.1)
    }

    pub fn label(&self, translations: &Translations) -> &str {
        self.title
            .as_deref()
            .unwrap_or_else(||
                self.slug_opt()
                    .unwrap_or(translations.video)
            )
    }

    /// Applies a simple heuristic to determine the "most recommended"
    /// version to watch. Roughly speaking, something that is not too
    /// high-res, not too low-res, but a nice balance between resource
    /// consumption and visual fidelity.
    pub fn recommended_version(&self) -> Option<&VideoFile> {
        // (fps_distance, height_distance, version)
        let mut version_candidate: Option<(f32, i32, &VideoFile)> = None;

        for version in &self.versions {
            if let Some(video_meta) = &version.video_meta {
                let fps_distance = (27.0 - video_meta.video_frame_rate).abs();
                let height_distance = (720 - video_meta.height as i32).abs();
                if version_candidate.is_none() || version_candidate.is_some_and(|version_tuple| {
                    fps_distance < version_tuple.0 || height_distance < version_tuple.1
                }) {
                    version_candidate = Some((fps_distance, height_distance, version));
                }
            }
        }

        version_candidate.map(|version_tuple| version_tuple.2)
    }

    fn new(path: String) -> Video {
        Video {
            description: None,
            errors: Vec::new(),
            links: Vec::new(),
            path,
            poster: None,
            release_date: None,
            sort_number: None,
            subtitles: Vec::new(),
            title: None,
            unlisted: false,
            versions: Vec::new()
        }
    }

    pub fn parent_path(&self) -> &Path {
        if self.path.is_empty() {
            Path::new("")
        } else {
            Path::new(&self.path).parent().unwrap()
        }
    }

    pub fn read_dir(context: &Context, hyper_dir: &HyperDir, site: &Site) -> Arc<Video> {
        let mut video = Video::new(hyper_dir.site_path(context));

        if let Some(manifest) = &hyper_dir.video_manifest {
            video.apply_manifest(context, manifest, site);
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

            let assets = context.cache.lock().unwrap().fetch_poster_assets(
                &file_meta,
                &file_name,
                &video.path,
                site.poster_aspect.numeric()
            );

            let poster_file = PosterFile::new(assets, None, file_meta, file_name);

            video.poster = Some(poster_file);
        }

        for subtitle_file in &hyper_dir.subtitle_files {
            video.subtitles.push(SubtitleFile::new(subtitle_file));
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

    // TODO: Terminate any scheduled/running background jobs associated with the video?
    pub fn unpersist(&self, context: &Context) {
        if self.path == "" {
            let manifest_path = context.site_dir.join("video.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(poster) = &self.poster {
                let poster_path = context.site_dir.join(&poster.file_name);
                let _ = fs::remove_file(&poster_path);
            }

            for version in &self.versions {
                let version_path = context.site_dir.join(&version.file_name);
                let _ = fs::remove_file(&version_path);
            }
        } else {
            let video_dir = context.site_dir.join(&self.path);
            let _ = checked_remove_dir_all(context, &video_dir);
        }
    }

    pub fn versions_desc_by_quality(&self) -> Vec<VideoFile> {
        let mut versions: Vec<VideoFile> = self.versions.iter().cloned().collect();
        versions.sort_unstable_by(|a, b| {
            match (&a.video_meta, &b.video_meta) {
                (Some(a_video_meta), Some(b_video_meta)) => {
                    if a_video_meta.height != b_video_meta.height {
                        b_video_meta.height.cmp(&a_video_meta.height)
                    } else if a_video_meta.video_frame_rate != b_video_meta.video_frame_rate {
                        b_video_meta.video_frame_rate.partial_cmp(&a_video_meta.video_frame_rate)
                            .unwrap_or_else(|| b.file_meta.size.cmp(&a.file_meta.size))
                    } else {
                        b.file_meta.size.cmp(&a.file_meta.size)
                    }
                }
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => b.file_meta.size.cmp(&a.file_meta.size)
            }
        });
        versions
    }

    pub fn write_manifest(&self, context: &Context) {
        let mut eno = String::new();

        for link in &self.links {
            eno.push_str(&link.to_eno());
        }

        if let Some(poster) = &self.poster {
            eno.push_str("poster:\n");

            if let Some(description) = &poster.description {
                eno.push_str(&format!("description = {}\n", description));
            }

            eno.push_str(&format!("file = {}\n", poster.file_name));
        }

        if let Some(release_date) = &self.release_date {
            eno.push_str(&format!("release_date: {}\n", release_date));
        }

        if let Some(sort_number) = &self.sort_number {
            eno.push_str(&format!("sort_number: {}\n", sort_number));
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

    pub fn vertical_resolution_label(&self) -> String {
        match &self.video_meta {
            Some(video_meta) => format!("{}p", video_meta.height),
            None => String::new()
        }
    }
}

impl VideoMeta {
    pub fn new(
        audio_codec: Option<String>,
        duration: f32,
        height: usize,
        video_codec: Option<String>,
        video_frame_rate: f32,
        width: usize
    ) -> VideoMeta {
        VideoMeta {
            audio_codec,
            duration,
            height,
            video_codec,
            video_frame_rate,
            width
        }
    }

    pub fn write_cache_manifest(
        &self,
        context: &Context,
        file_meta: &FileMeta,
        file_name: &str,
        path: &str
    ) {
        let version_path = format!("{}/{}", path, file_name);

        let cache_item = CacheItem::VideoMeta(VideoMetaCached {
            file_meta: file_meta.clone(),
            path: version_path.clone(),
            video_meta: self.clone()
        });

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let cache_item_file_name = format!("{}.bincode", hash(&version_path));
        let cache_item_path = context.cache_dir.join(cache_item_file_name);
        
        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}
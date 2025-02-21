// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use serde_derive::{Deserialize, Serialize};

use crate::{
    CacheItem,
    Collection,
    Context,
    FileMeta,
    HyperDir,
    HyperUrl,
    Link,
    MergedOptions,
    Playlist,
    PosterFile,
    Site,
    SubtitleFile,
    Translations,
    VideoMetaCached
};
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::{checked_remove_dir_all, hash};

const VIDEO_OPTIONS: &[&str] = &[
    "description",
    "download",
    "embedding",
    "link",
    "offline",
    "poster",
    "release_date",
    "sort_number",
    "title",
    "unlisted"
];

#[derive(Clone, Debug)]
pub struct Video {
    pub description: Option<String>,
    pub download: Option<bool>,
    pub embedding: Option<bool>,
    pub errors: Vec<String>,
    pub links: Vec<Link>,
    pub offline: bool,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoOrder {
    Default,
    ReleaseDateAsc,
    ReleaseDateDesc,
    SortNumberAsc,
    SortNumberDesc,
    TitleAsc,
    TitleDesc
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

        for element in document.elements() {
            match element.key() {
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
                "download" => 'download: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.download = Some(true);
                                    }
                                    "disabled" => {
                                        self.download = Some(false);
                                    }
                                    other => {
                                        let message = format!("The download option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }

                                break 'download;
                            }
                        }
                    }

                    let message = "download needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'download: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "embedding" => 'embedding: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.embedding = Some(true);
                                    }
                                    "disabled" => {
                                        self.embedding = Some(false);
                                    }
                                    other => {
                                        let message = format!("The embedding option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }

                                break 'embedding;
                            }
                        }
                    }

                    let message = "embedding needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'embedding: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
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
                "poster" => 'poster: {
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
                                        let message = "The key/name of this attribute was not recognized, only 'description' and 'file' are recognized inside an poster field";
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
                                    site.poster_aspect.numeric()
                                );

                                let poster = PosterFile::new(assets, description, file_meta, file_name);
                                self.poster = Some(poster);
                            }

                            break 'poster;
                        }
                    }

                    let message = "poster needs to be provided as a field with attributes, e.g.:\n\nposter:\ndescription = Denis, giggling uncontrollably\nfile = denis.jpg";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "release_date" => 'title: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                                    Ok(date) => {
                                        self.release_date = Some(date);
                                    }
                                    Err(err) => {
                                        let message = format!("Invalid date value '{value}': {err}");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'title;
                        }
                    }

                    let message = "release_date needs to be provided as a field with a date value in the format YYYY-MM-DD, e.g.: 'release_date: 2025-02-17'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
                }
                "sort_number" => 'title: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value.parse::<i32>() {
                                    Ok(number) => {
                                        self.sort_number = Some(number);
                                    }
                                    Err(err) => {
                                        let message = format!("Invalid number value '{value}': {err}");
                                        let error = element_error(element, manifest_path, &message);
                                        self.errors.push(error);
                                    }
                                }
                            }

                            break 'title;
                        }
                    }

                    let message = "sort_number needs to be provided as a field with a number value, e.g.: 'sort_number: 13'";
                    let error = element_error(element, manifest_path, message);
                    self.errors.push(error);
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

                    let message = "title needs to be provided as a field with a value, e.g.: 'title: Denis Supercut'";
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
                other => {
                    let message = not_supported_error("video.eno", other, &[VIDEO_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.errors.push(error);
                }
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

    // Returns a comparator function according to the passed [VideoOrder]
    pub fn cmp_by_video_order(video_order: &VideoOrder) -> fn(&Arc<Video>, &Arc<Video>) -> Ordering {
        match video_order {
            VideoOrder::Default => Video::cmp_by_sort_number_then_release_date_asc,
            VideoOrder::ReleaseDateAsc => Video::cmp_by_release_date_asc,
            VideoOrder::ReleaseDateDesc => Video::cmp_by_release_date_desc,
            VideoOrder::SortNumberAsc => Video::cmp_by_sort_number_asc,
            VideoOrder::SortNumberDesc => Video::cmp_by_sort_number_desc,
            VideoOrder::TitleAsc => Video::cmp_by_title_or_slug_asc,
            VideoOrder::TitleDesc => Video::cmp_by_title_or_slug_desc
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

    pub fn new(path: String) -> Video {
        Video {
            description: None,
            download: None,
            embedding: None,
            errors: Vec::new(),
            links: Vec::new(),
            offline: false,
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

    /// The merged options constructed from all ancestors, or in the case of
    //  a site-level video just the general default options.
    pub fn parent_options(&self, site: &Site) -> MergedOptions {
        if self.path.is_empty() {
            MergedOptions::default()
        } else {
            let parent_path = Path::new(&self.path).parent().unwrap();
            site.get_merged_options_unchecked(parent_path.to_str().unwrap())
        }
    }

    // TODO: Maybe this should better be parent_path_unchecked and we only use
    //       the current 'else' branch? Calling this on a root node currently
    //       returns a semi-correct/meaningful result.
    //       (also applies to collection/playlist)
    pub fn parent_path(&self) -> &Path {
        if self.path.is_empty() {
            Path::new("")
        } else {
            Path::new(&self.path).parent().unwrap()
        }
    }

    pub fn public(&self) -> bool {
        !self.offline && !self.unlisted
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

        if let Some(download) = self.download {
            eno.push_str(&format!("download: {}\n", if download { "enabled" } else { "disabled" }));
        }

        if let Some(embedding) = self.embedding {
            eno.push_str(&format!("embedding: {}\n", if embedding { "enabled" } else { "disabled" }));
        }

        if self.offline {
            eno.push_str("offline\n");
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
            eno.push_str(&format!("-- description\n{}\n-- description\n", description));
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

impl VideoOrder {
    pub fn from_key(key: &str) -> Result<VideoOrder, String> {
        match key {
            "default" => Ok(VideoOrder::Default),
            "release_date_asc" => Ok(VideoOrder::ReleaseDateAsc),
            "release_date_desc" => Ok(VideoOrder::ReleaseDateDesc),
            "sort_number_asc" => Ok(VideoOrder::SortNumberAsc),
            "sort_number_desc" => Ok(VideoOrder::SortNumberDesc),
            "title_asc" => Ok(VideoOrder::TitleAsc),
            "title_desc" => Ok(VideoOrder::TitleDesc),
            _ => {
                // TODO: Translate
                let message = format!("Unsupported order key '{key}' (supported ones are 'default', 'release_date_asc', 'release_date_desc', 'sort_number_asc', 'sort_number_desc', 'title_asc' and 'title_desc')");
                Err(message)
            }
        }
    }

    pub fn key(&self) -> &str {
        match self {
            VideoOrder::Default => "default",
            VideoOrder::ReleaseDateAsc => "release_date_asc",
            VideoOrder::ReleaseDateDesc => "release_date_desc",
            VideoOrder::SortNumberAsc => "sort_number_asc",
            VideoOrder::SortNumberDesc => "sort_number_desc",
            VideoOrder::TitleAsc => "title_asc",
            VideoOrder::TitleDesc => "title_desc"
        }
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use icu_collator::CollatorBorrowed;
use url::Url;

use crate::{
    AudioFile,
    Context,
    FileMeta,
    HyperDir,
    InheritedOptions,
    Link,
    PlatformIntegration,
    PosterFile,
    Site,
    SitePath,
    SubtitleFile,
    Translations,
    VideoFile,
    VideoOrder
};
use language_tags::LanguageTag;
use crate::manifest::MAX_SYNOPSIS_CHARS;
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::{
    checked_remove_dir_all,
    supported_subtitles_extension
};

const VIDEO_OPTIONS: &[&str] = &[
    "copy_link",
    "description",
    "download",
    "embedding",
    "link",
    "offline",
    "platform_integration",
    "poster",
    "release_date",
    "sort_number",
    "subtitles",
    "synopsis",
    "title",
    "unlisted"
];

#[derive(Clone, Debug)]
pub struct Video {
    pub audio_files: Vec<AudioFile>,
    copy_link: Option<bool>,
    pub description: Option<String>,
    /// Errors related to files within the video directory, outside of
    /// the video manifest which uses `manifest_errors`.
    pub dir_errors: Vec<String>,
    download: Option<bool>,
    embedding: Option<bool>,
    /// Some options of a video (e.g. copy_link, embedding and unlisted)
    /// are affected by both the local settings in the manifest as well as
    /// the inherited state from ancestors in the site tree. This stores
    /// the options interited by this video from its ancestors.
    inherited_options: InheritedOptions,
    pub links: Vec<Link>,
    /// Syntactical and semantic errors inside the manifest for displaying to
    /// the user as well as to prevent programmatic updates to the site and
    /// its manifest while such errors are present.
    pub manifest_errors: Vec<String>,
    offline: bool,
    platform_integration: Option<PlatformIntegration>,
    pub poster: Option<PosterFile>,
    pub release_date: Option<NaiveDate>,
    pub site_path: SitePath,
    pub sort_number: Option<i32>,
    pub subtitles: Vec<SubtitleFile>,
    pub synopsis: Option<String>,
    pub title: Option<String>,
    /// Unlisted videos are not linked from the site anywhere, one needs to
    /// know the link to access them. This property propagates downwards in
    /// in the site tree once it is set (e.g. on a collection or playlist).
    unlisted: bool,
    pub video_files: Vec<VideoFile>
}

impl Video {
    pub fn all_metadata_computed(&self) -> bool {
        (self.poster.is_none() || self.poster.as_ref().is_some_and(|poster| poster.assets.is_some())) &&
        self.audio_files.iter().all(|audio_file| audio_file.audio_meta.is_some()) &&
        self.video_files.iter().all(|video_file| video_file.video_meta.is_some())
    }

    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path, site: &Site) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.manifest_errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return
            }
        };

        for element in document.elements() {
            match element.key() {
                "copy_link" => 'copy_link: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.copy_link = Some(true);
                                    }
                                    "disabled" => {
                                        self.copy_link = Some(false);
                                    }
                                    other => {
                                        let message = format!("The copy_link option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'copy_link;
                            }
                        }
                    }

                    let message = "copy_link needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'copy_link: disabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
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
                        self.manifest_errors.push(error);
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
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'download;
                            }
                        }
                    }

                    let message = "download needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'download: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
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
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'embedding;
                            }
                        }
                    }

                    let message = "embedding needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'embedding: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "link" => 'link: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match Url::parse(value) {
                                    Ok(_) => {
                                        let link = Link::new(false, None, false, value);
                                        self.links.push(link);
                                    }
                                    Err(err) => {
                                        let message = format!("The url supplied for the link seems to be malformed ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
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
                                            match Url::parse(value) {
                                                Ok(_) => url = Some(value),
                                                Err(err) => {
                                                    let message = format!("The url supplied for the link seems to be malformed ({err})");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.manifest_errors.push(error);
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
                                                    self.manifest_errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    other => {
                                        let message = format!("The attribute '{other}' is not recognized here (supported attributes are 'label', 'url' and 'verification'");
                                        let error = attribute_error(attribute, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            if let Some(url) = url {
                                let link = Link::new(hidden, label, rel_me, url);
                                self.links.push(link);
                            } else {
                                let message = "The link option must supply an url attribute at least, e.g.:\n\nlink:\nurl = https://example.com";
                                let error = element_error(element, manifest_path, message);
                                self.manifest_errors.push(error);
                            }

                            break 'link;
                        }
                    }

                    let message = "link must be provided as a basic field with a value (e.g. 'link: https://example.com') or in its extended form as a field with attributes, e.g.:\n\nlink:\nurl = https://example.com\nlabel = Example";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "offline" => {
                    if element.is_flag() {
                        self.offline = true;
                    } else {
                        let message = "offline needs to be provided as a flag, that is, exactly as 'offline' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "platform_integration" => 'platform_integration: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match PlatformIntegration::from_key(value) {
                                    Some(platform_integration) => {
                                        self.platform_integration = Some(platform_integration);
                                    }
                                    None => {
                                        let message = format!("The platform_integration option only supports the value 'link_previews', 'content_embeds' or 'disabled'. The given value '{value}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'platform_integration;
                            }
                        }
                    }

                    let message = "platform_integration needs to be provided as a field with the value 'link_previews', 'content_embeds' or 'disabled', e.g.: 'platform_integration: link_previews'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
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
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                    _ => {
                                        let message = "The key/name of this attribute was not recognized, only 'description' and 'file' are recognized inside an poster field";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            if let Some((absolute_path, file_name)) = file {
                                let file_meta = FileMeta::new(&absolute_path);

                                let assets;
                                let poster_meta;

                                {
                                    // Obtain lock on the cache, immediately drop it afterwards
                                    let cache = context.cache.lock().unwrap();

                                    assets = cache.fetch_poster_assets(
                                        &file_meta,
                                        &file_name,
                                        site.poster_aspect.numeric(),
                                        &self.site_path
                                    );

                                    poster_meta = cache.fetch_poster_source(
                                        &file_meta,
                                        &file_name,
                                        &self.site_path
                                    );
                                }

                                let poster = PosterFile::new(
                                    assets,
                                    description,
                                    file_meta,
                                    file_name,
                                    poster_meta
                                );
                                self.poster = Some(poster);
                            }

                            break 'poster;
                        }
                    }

                    let message = "poster needs to be provided as a field with attributes, e.g.:\n\nposter:\ndescription = Denis, giggling uncontrollably\nfile = denis.jpg";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
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
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            break 'title;
                        }
                    }

                    let message = "release_date needs to be provided as a field with a date value in the format YYYY-MM-DD, e.g.: 'release_date: 2025-02-17'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
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
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            break 'title;
                        }
                    }

                    let message = "sort_number needs to be provided as a field with a number value, e.g.: 'sort_number: 13'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "subtitles" => 'subtitles: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(attributes) = field.attributes() {
                            let mut file = None;
                            let mut label = None;
                            let mut language = None;

                            for attribute in attributes {
                                match attribute.key() {
                                    "file" => {
                                        if let Some(value) = attribute.value() {
                                            let absolute_path = manifest_path.parent().unwrap().join(value);

                                            if absolute_path.exists() {
                                                if supported_subtitles_extension(value) {
                                                    file = Some(value.to_string());
                                                } else {
                                                    let message = format!("Please supply subtitles in .srt or .vtt format, other formats are not (yet) supported ({value}).");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.manifest_errors.push(error);
                                                }
                                            } else {
                                                let message = format!("The referenced file was not found ({})", absolute_path.display());
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                    "label" => {
                                        if let Some(value) = attribute.value() {
                                            label = Some(value.to_string());
                                        }
                                    }
                                    "language" => {
                                        if let Some(value) = attribute.value() {
                                            match LanguageTag::parse(value) {
                                                Ok(language_tag) => {
                                                    match language_tag.validate() {
                                                        Ok(()) => {
                                                            language = Some(language_tag.to_string())
                                                        }
                                                        Err(err) => {
                                                            let message = format!("The language code supplied for the subtitles is not recognized ({err})");
                                                            let error = attribute_error(attribute, manifest_path, &message);
                                                            self.manifest_errors.push(error);
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    let message = format!("The language code supplied for the subtitles is not properly formatted ({err})");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.manifest_errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    other => {
                                        let message = format!("The attribute '{other}' is not recognized here (supported attributes are 'file', 'label' and 'language'");
                                        let error = attribute_error(attribute, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            if let Some(file) = file {
                                let subtitles = SubtitleFile::from_manifest(file, label, language);
                                self.subtitles.push(subtitles);
                            } else {
                                let message = "The subtitles option must supply a file attribute at least, e.g.:\n\nsubtitles:\nfile = en.srt";
                                let error = element_error(element, manifest_path, message);
                                self.manifest_errors.push(error);
                            }

                            break 'subtitles;
                        }
                    }

                    let message = "subtitles must be provided as a field with attributes, e.g.:\n\nsubtitles:\nfile = en.srt\nlabel = English (closed captions)\nlanguage = en";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "synopsis" => {
                    if let Ok(embed) = element.as_embed() {
                        if let Some(value) = embed.value() {
                            let synopsis_chars = value.chars().count();

                            if synopsis_chars <= MAX_SYNOPSIS_CHARS {
                                self.synopsis = Some(value.to_string());
                            } else {
                                let message = format!("Synopsis is too long ({synopsis_chars}/{MAX_SYNOPSIS_CHARS} characters)");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        } else {
                            self.synopsis = None;
                        }
                    } else {
                        let message = "The 'synopsis' option needs to be provided as an embed, e.g.:\n-- synopsis\nA synopsis text\n--synopsis";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
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

                    let message = "title needs to be provided as a field with a value, e.g.: 'title: Denis Supercut'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "unlisted" => {
                    if element.is_flag() {
                        self.unlisted = true;
                    } else {
                        let message = "unlisted needs to be provided as a flag, that is, exactly as 'unlisted' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                other => {
                    let message = not_supported_error("video.eno", other, &[VIDEO_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.manifest_errors.push(error);
                }
            }
        }
    }

    /// When we convert between types (collection, playlist, video) we pass
    /// over existing inherited options using this method.
    pub fn copy_inherited_options(&self) -> InheritedOptions {
        self.inherited_options.clone()
    }

    // Videos without an earliest modified date date are sorted to the back
    pub fn cmp_by_earliest_modified_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        _: &CollatorBorrowed
    ) -> Ordering {
        let a_earliest_modified = a.video_files
            .iter()
            .map(|video_file| video_file.file_meta.modified)
            .min();

        let b_earliest_modified = b.video_files
            .iter()
            .map(|video_file| video_file.file_meta.modified)
            .min();

        match (a_earliest_modified, b_earliest_modified) {
            (Some(a_modified), Some(b_modified)) => a_modified.cmp(&b_modified),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without an earliest modified date date are sorted to the back
    pub fn cmp_by_earliest_modified_desc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        _: &CollatorBorrowed
    ) -> Ordering {
        let a_earliest_modified = a.video_files
            .iter()
            .map(|video_file| video_file.file_meta.modified)
            .min();

        let b_earliest_modified = b.video_files
            .iter()
            .map(|video_file| video_file.file_meta.modified)
            .min();

        match (a_earliest_modified, b_earliest_modified) {
            (Some(a_modified), Some(b_modified)) => b_modified.cmp(&a_modified),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without release date are sorted to the back
    pub fn cmp_by_release_date_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        match (&a.release_date, &b.release_date) {
            (Some(a_release_date), Some(b_release_date)) => {
                match a_release_date.cmp(b_release_date) {
                    Ordering::Equal => Video::cmp_by_earliest_modified_asc(a, b, collator),
                    ordering => ordering
                }
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without release date are sorted to the back
    pub fn cmp_by_release_date_desc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        match (&a.release_date, &b.release_date) {
            (Some(a_release_date), Some(b_release_date)) => {
                match b_release_date.cmp(a_release_date) {
                    Ordering::Equal => Video::cmp_by_earliest_modified_desc(a, b, collator),
                    ordering => ordering
                }
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without title are sorted to the front
    pub fn cmp_by_slug_then_title_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => collator.compare(a_title, b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => collator.compare(a.slug_unchecked(), b.slug_unchecked())
        }
    }

    // Videos without title are sorted to the front
    pub fn cmp_by_slug_then_title_desc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => collator.compare(b_title, a_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => collator.compare(b.slug_unchecked(), a.slug_unchecked())
        }
    }

    // Videos without sort number are sorted to the back
    pub fn cmp_by_sort_number_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        _: &CollatorBorrowed
    ) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without sort number are sorted to the back
    pub fn cmp_by_sort_number_desc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        _: &CollatorBorrowed
    ) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => b_sort_number.cmp(a_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    }

    // Videos without sort number and without release date are sorted to the back
    pub fn cmp_by_sort_number_then_release_date_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        match (&a.sort_number, &b.sort_number) {
            (Some(a_sort_number), Some(b_sort_number)) => a_sort_number.cmp(b_sort_number),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Video::cmp_by_release_date_asc(a, b, collator)
        }
    }

    // Videos with title and videos with only a slug are sorted together
    pub fn cmp_by_title_or_slug_asc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        let a_title = a.title.as_deref().unwrap_or_else(|| a.slug_unchecked());
        let b_title = b.title.as_deref().unwrap_or_else(|| b.slug_unchecked());

        collator.compare(a_title, b_title)
    }

    // Videos with title and videos with only a slug are sorted together
    pub fn cmp_by_title_or_slug_desc(
        a: &Arc<Video>,
        b: &Arc<Video>,
        collator: &CollatorBorrowed
    ) -> Ordering {
        let a_title = a.title.as_deref().unwrap_or_else(|| a.slug_unchecked());
        let b_title = b.title.as_deref().unwrap_or_else(|| b.slug_unchecked());

        collator.compare(b_title, a_title)
    }

    // Returns a comparator function according to the passed [VideoOrder]
    pub fn cmp_by_video_order(
        video_order: &VideoOrder
    ) -> fn(&Arc<Video>, &Arc<Video>, &CollatorBorrowed) -> Ordering {
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

    /// Get the copy_link setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn copy_link(&self) -> bool {
        self.copy_link.unwrap_or(self.inherited_options.copy_link)
    }

    /// Get the copy_link setting as inherited from ancestors.
    pub fn copy_link_inherited(&self) -> bool {
        self.inherited_options.copy_link
    }

    /// Get the copy_link setting as set locally on the video.
    pub fn copy_link_local(&self) -> Option<bool> {
        self.copy_link
    }

    /// Set the copy_link setting locally on the video.
    pub fn copy_link_local_set(&mut self, enabled_option: Option<bool>) {
        self.copy_link = enabled_option;
    }

    /// Get the download setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn download(&self) -> bool {
        self.download.unwrap_or(self.inherited_options.download)
    }

    /// Get the download setting as inherited from ancestors.
    pub fn download_inherited(&self) -> bool {
        self.inherited_options.download
    }

    /// Get the download setting as set locally on the video.
    pub fn download_local(&self) -> Option<bool> {
        self.download
    }

    /// Set the download setting locally on the video.
    pub fn download_local_set(&mut self, enabled_option: Option<bool>) {
        self.download = enabled_option;
    }

    /// Get the embedding setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn embedding(&self) -> bool {
        self.embedding.unwrap_or(self.inherited_options.embedding)
    }

    /// Get the embedding setting as inherited from ancestors.
    pub fn embedding_inherited(&self) -> bool {
        self.inherited_options.embedding
    }

    /// Get the embedding setting as set locally on the video.
    pub fn embedding_local(&self) -> Option<bool> {
        self.embedding
    }

    /// Set the embedding setting locally on the video.
    pub fn embedding_local_set(&mut self, enabled_option: Option<bool>) {
        self.embedding = enabled_option;
    }

    /// Determines if the video needs to render embeds. This information is
    /// used to check whether we need to write embed css/js assets to the
    /// build.
    pub fn embeds_requested(&self) -> bool {
        self.embedding() || self.platform_integration().content_embeds()
    }

    /// Return the video file with the maximum pixel count (horizontal ×
    /// vertical pixels), or in case of a tie, the video file that
    /// additionally has the larger file size.
    pub fn highest_quality_video_file(&self) -> Option<&VideoFile> {
        struct Candidate<'a> {
            pixel_count: usize,
            video_file: &'a VideoFile
        }

        let mut candidate: Option<Candidate> = None;

        for video_file in &self.video_files {
            if let Some(video_meta) = &video_file.video_meta {
                let pixel_count = video_meta.width * video_meta.height;
                if candidate.is_none() || candidate.as_ref().is_some_and(|candidate| {
                        pixel_count > candidate.pixel_count ||
                        (pixel_count == candidate.pixel_count && video_file.file_meta.size > candidate.video_file.file_meta.size)
                }) {
                    candidate = Some(Candidate {
                        pixel_count,
                        video_file
                    });
                }
            } else if candidate.is_none() {
                candidate = Some(Candidate {
                    pixel_count: 0,
                    video_file
                });
            }
        }

        candidate.map(|candidate| candidate.video_file)
    }

    /// Sets new inherited options on the video.
    pub fn inherited_options_set(&mut self, inherited_options: InheritedOptions) {
        self.inherited_options = inherited_options;
    }

    pub fn new(inherited_options: InheritedOptions, site_path: SitePath) -> Video {
        Video {
            audio_files: Vec::new(),
            copy_link: None,
            description: None,
            dir_errors: Vec::new(),
            download: None,
            embedding: None,
            inherited_options,
            links: Vec::new(),
            manifest_errors: Vec::new(),
            offline: false,
            platform_integration: None,
            poster: None,
            release_date: None,
            site_path,
            sort_number: None,
            subtitles: Vec::new(),
            synopsis: None,
            title: None,
            unlisted: false,
            video_files: Vec::new()
        }
    }

    /// Get the offline setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn offline(&self) -> bool {
        self.offline || self.inherited_options.offline
    }

    /// Get the offline setting as inherited from ancestors.
    pub fn offline_inherited(&self) -> bool {
        self.inherited_options.offline
    }

    /// Get the offline setting as set locally on the video.
    pub fn offline_local(&self) -> bool {
        self.offline
    }

    /// Set the offline setting locally on the video.
    pub fn offline_local_set(&mut self, enabled: bool) {
        self.offline = enabled;
    }

    // Calling this on a root node returns the same site path again
    pub fn parent_site_path_unchecked(&self) -> SitePath {
        self.site_path.parent_site_path_unchecked()
    }

    /// Get the platform_integration setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn platform_integration(&self) -> PlatformIntegration {
        self.platform_integration.unwrap_or(self.inherited_options.platform_integration)
    }

    /// Get the platform_integration setting as inherited from ancestors.
    pub fn platform_integration_inherited(&self) -> PlatformIntegration {
        self.inherited_options.platform_integration
    }

    /// Get the platform_integration setting as set locally on the video.
    pub fn platform_integration_local(&self) -> Option<PlatformIntegration> {
        self.platform_integration
    }

    /// Set the platform_integration setting locally on the video.
    pub fn platform_integration_local_set(&mut self, platform_integration_option: Option<PlatformIntegration>) {
        self.platform_integration = platform_integration_option;
    }

    /// A video is public in its local context if it is neither offline
    /// nor unlisted according to its own settings. This does
    /// (intentionally) not take into account publicity settings inherited
    /// from ancestors.
    pub fn public_local(&self) -> bool {
        !self.offline && !self.unlisted
    }

    pub fn read_dir(
        context: &Context,
        hyper_dir: &HyperDir,
        inherited_options: InheritedOptions,
        site: &Site
    ) -> Arc<Video> {
        let site_path = hyper_dir.site_path(context);
        let mut video = Video::new(inherited_options, site_path);

        if let Some(manifest) = &hyper_dir.video_manifest {
            video.apply_manifest(context, manifest, site);
        }

        if !hyper_dir.dirs.is_empty() {
            let folder_names = hyper_dir.dirs
                .iter()
                .map(|dir| format!("'{}'", dir.file_name().unwrap().to_string_lossy()))
                .collect::<Vec<String>>()
                .join(", ");

            video.dir_errors.push(format!("Ignoring folders {folder_names} inside video folder {} (only files may be placed inside a video folder).", hyper_dir.path.display()));
        }

        // TODO: Some strategy what we do with multiple images, e.g.: A heuristic
        // to pick the most probable poster, using additional ones, etc.
        if !hyper_dir.image_files.is_empty() && video.poster.is_none() {
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let assets;
            let poster_meta;

            {
                // Obtain lock on the cache, immediately drop it afterwards
                let cache = context.cache.lock().unwrap();

                assets = cache.fetch_poster_assets(
                    &file_meta,
                    &file_name,
                    site.poster_aspect.numeric(),
                    &video.site_path
                );

                poster_meta = cache.fetch_poster_source(
                    &file_meta,
                    &file_name,
                    &video.site_path
                );
            }

            let poster_file = PosterFile::new(
                assets,
                None,
                file_meta,
                file_name,
                poster_meta
            );

            video.poster = Some(poster_file);
        }

        for path in &hyper_dir.subtitle_files {
            if !video.subtitles
                .iter()
                .any(|subtitle_file| *subtitle_file.file_name == *path.file_name().unwrap()) {
                video.subtitles.push(SubtitleFile::from_path(path));
            }
        }

        for video_file_path in &hyper_dir.video_files {
            let file_meta = FileMeta::new(video_file_path);
            let file_name = video_file_path.file_name().unwrap().to_string_lossy().to_string();

            let version_path = format!("{}{}", &video.site_path.normalized(), file_name);
            let video_meta = context.cache.lock().unwrap().video_meta
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
                .map(|cached| cached.video_meta.clone());

            video.video_files.push(VideoFile::new(file_meta, file_name, video_meta));
        }

        for audio_file_path in &hyper_dir.audio_files {
            let file_meta = FileMeta::new(audio_file_path);
            let file_name = audio_file_path.file_name().unwrap().to_string_lossy().to_string();

            let version_path = format!("{}{}", &video.site_path.normalized(), file_name);
            let audio_meta = context.cache.lock().unwrap().audio_meta
                .iter()
                .find(|cached| cached.file_meta == file_meta && cached.path == version_path)
                .map(|cached| cached.audio_meta.clone());

            video.audio_files.push(AudioFile::new(audio_meta, file_meta, file_name));
        }

        if !hyper_dir.errors.is_empty() {
            let mut error = String::from("Directories and/or files inside this video directory have issues:");

            for message in &hyper_dir.errors {
                error.push_str(&format!("\n- {}", message));
            }

            video.dir_errors.push(error);
        }

        Arc::new(video)
    }

    /// Currently picks the audio file with the highest compatibility, if
    /// there is any audio file.
    pub fn recommended_audio_file(&self) -> Option<&AudioFile> {
        self.audio_files.iter().find(|audio_file| {
                audio_file.audio_meta.as_ref().is_some_and(|audio_meta|
                    audio_meta.codec.as_ref().is_some_and(|codec| codec == "mp3")
                )
            })
            .or_else(|| {
                self.audio_files.iter().find(|audio_file| {
                    audio_file.audio_meta.as_ref().is_some_and(|audio_meta|
                        audio_meta.codec.as_ref().is_some_and(|codec| codec == "opus")
                    )
                })
            })
            .or_else(|| self.audio_files.first())
    }

    /// Applies a simple heuristic to determine the "most recommended"
    /// video file to watch. Roughly speaking, something that is not too
    /// high-res, not too low-res, but a nice balance between resource
    /// consumption and visual fidelity.
    pub fn recommended_video_file(&self) -> Option<&VideoFile> {
        const IDEAL_FRAME_RATE: f32 = 27.0;
        const IDEAL_HEIGHT: usize = 720;

        struct Candidate<'a> {
            frame_rate_distance: f32,
            height_distance: u32,
            video_file: &'a VideoFile
        }

        let mut candidate_option: Option<Candidate> = None;

        for video_file in &self.video_files {
            if let Some(video_meta) = &video_file.video_meta {
                let frame_rate_distance = (IDEAL_FRAME_RATE - video_meta.video_frame_rate).abs();
                let height_distance = (IDEAL_HEIGHT as i32 - video_meta.height as i32).unsigned_abs();

                if candidate_option.is_none() || candidate_option.as_ref().is_some_and(|candidate| {
                    frame_rate_distance <= candidate.frame_rate_distance &&
                    height_distance <= candidate.height_distance
                }) {
                    candidate_option = Some(Candidate {
                        frame_rate_distance,
                        height_distance,
                        video_file
                    });
                }
            }
        }

        candidate_option.map(|candidate| candidate.video_file)
    }

    pub fn slug(&self) -> Option<&str> {
        self.site_path.last_component()
    }

    pub fn slug_unchecked(&self) -> &str {
        self.site_path.last_component_unchecked()
    }

    pub fn title_or_slug_or_generic_label(&self, translations: &Translations) -> &str {
        match &self.title {
            Some(title) => title,
            None => self.slug().unwrap_or(translations.video)
        }
    }

    /// Call this when you need a title or slug and you're sure that a
    /// fallback slug will always be available. For instance: The videos in a
    /// feed are always children of a collection or playlist (because feeds
    /// are only avilable for those two), so in that context this is the best
    /// method to get a title for videos.
    pub fn title_or_slug_unchecked(&self) -> &str {
        match &self.title {
            Some(title) => title,
            None => self.slug_unchecked()
        }
    }

    /// Get the unlisted setting resolved in the context of the site tree
    /// (= as set locally on the video or as inherited from an ancestor).
    pub fn unlisted(&self) -> bool {
        self.unlisted || self.inherited_options.unlisted
    }

    /// Get the unlisted setting as inherited from ancestors.
    pub fn unlisted_inherited(&self) -> bool {
        self.inherited_options.unlisted
    }

    /// Get the unlisted setting as set locally on the video.
    pub fn unlisted_local(&self) -> bool {
        self.unlisted
    }

    /// Set the unlisted setting locally on the video.
    pub fn unlisted_local_set(&mut self, enabled: bool) {
        self.unlisted = enabled;
    }

    // TODO: Terminate any scheduled/running background jobs associated with the video?
    pub fn unpersist(&self, context: &Context) {
        if self.site_path.is_root_path() {
            let manifest_path = context.site_dir.join("video.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(poster) = &self.poster {
                let poster_path = context.site_dir.join(&poster.file_name);
                let _ = fs::remove_file(&poster_path);
            }

            for video_file in &self.video_files {
                let version_path = context.site_dir.join(&video_file.file_name);
                let _ = fs::remove_file(&version_path);
            }
        } else {
            let video_dir = context.site_dir.join(self.site_path.filesystem_relative());
            let _ = checked_remove_dir_all(context, &video_dir);
        }
    }

    pub fn video_files_asc_by_fidelity(&self) -> Vec<VideoFile> {
        let mut video_files: Vec<VideoFile> = self.video_files.clone();
        video_files.sort_unstable_by(|a, b| VideoFile::cmp_by_fidelity_desc(b, a));
        video_files
    }

    pub fn video_files_desc_by_fidelity(&self) -> Vec<VideoFile> {
        let mut video_files: Vec<VideoFile> = self.video_files.clone();
        video_files.sort_unstable_by(|a, b| VideoFile::cmp_by_fidelity_desc(a, b));
        video_files
    }

    pub fn write_manifest(&self, context: &Context) {
        let mut eno = String::new();

        for link in &self.links {
            eno.push_str(&link.to_eno());
        }

        if let Some(copy_link) = self.copy_link {
            eno.push_str(&format!("copy_link: {}\n", if copy_link { "enabled" } else { "disabled" }));
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

        if let Some(platform_integration) = self.platform_integration {
            eno.push_str(&format!("platform_integration: {}\n", platform_integration.key()));
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

        if let Some(poster) = &self.poster {
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str("poster:\n");

            if let Some(description) = &poster.description {
                eno.push_str(&format!("description = {}\n", description));
            }

            eno.push_str(&format!("file = {}\n", poster.file_name));
        }

        for subtitle_file in &self.subtitles {
            if !subtitle_file.explicitly_defined { continue; }
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str("subtitles:\n");
            eno.push_str(&format!("file = {}\n", subtitle_file.file_name));
            eno.push_str(&format!("label = {}\n", subtitle_file.label));

            if let Some(language) = &subtitle_file.language {
                eno.push_str(&format!("language = {}\n", language));
            }
        }

        if let Some(description) = &self.description {
            if !eno.is_empty() { eno.push('\n'); }
            eno.push_str(&format!("-- description\n{}\n-- description\n", description));
        }

        if let Some(synopsis) = &self.synopsis {
            if !eno.is_empty() { eno.push('\n'); }
            eno.push_str(&format!("-- synopsis\n{}\n-- synopsis\n", synopsis));
        }

        let path = context.site_dir.join(self.site_path.filesystem_relative()).join("video.eno");
        fs::write(path, eno).unwrap();
    }
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use slug::slugify;

use crate::{
    AudioFile,
    Context,
    SitePath,
    SubtitleFormat,
    VideoFile
};
use crate::util::{
    supported_font_extension,
    supported_image_extension,
    valid_slug_chars
};

/// Convenience wrapper to generically pre-scan directories for
/// repeating/required patterns used in hyper8's folder hierarchy.
#[derive(Debug)]
pub struct HyperDir {
    pub audio_files: Vec<PathBuf>,
    pub collection_manifest: Option<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub errors: Vec<String>,
    pub font_files: Vec<PathBuf>,
    pub image_files: Vec<PathBuf>,
    pub path: PathBuf,
    pub playlist_manifest: Option<PathBuf>,
    pub site_manifest: Option<PathBuf>,
    pub subtitle_files: Vec<HyperSubtitleFile>,
    pub video_files: Vec<PathBuf>,
    pub video_manifest: Option<PathBuf>
}

#[derive(Debug)]
pub struct HyperSubtitleFile {
    pub format: SubtitleFormat,
    pub path: PathBuf
}

impl HyperDir {
    fn new(path: &Path) -> HyperDir {
        HyperDir {
            audio_files: Vec::new(),
            collection_manifest: None,
            dirs: Vec::new(),
            errors: Vec::new(),
            font_files: Vec::new(),
            image_files: Vec::new(),
            path: path.to_owned(),
            playlist_manifest: None,
            site_manifest: None,
            subtitle_files: Vec::new(),
            video_files: Vec::new(),
            video_manifest: None
        }
    }

    pub fn read(path: &Path) -> HyperDir {
        let mut hyper_dir = HyperDir::new(path);

        if let Ok(dir_entries) = path.read_dir() {
            for dir_entry_result in dir_entries {
                if let Ok(dir_entry) = dir_entry_result {
                    if let Ok(file_type) = dir_entry.file_type() {
                        let file_name = dir_entry.file_name().to_string_lossy().to_string();

                        if file_name.starts_with('.') { continue }

                        if file_type.is_dir() {
                            if valid_slug_chars(&file_name) {
                                hyper_dir.dirs.push(dir_entry.path());
                            } else {
                                let slugified = slugify(&file_name);
                                let error = format!("Ignoring directory '{file_name}' because its name is invalid (try to only use alphanumeric characters, dash and underscore, e.g. '{slugified}')");
                                hyper_dir.errors.push(error);
                            }
                        } else if file_type.is_file() {
                            match file_name.as_ref() {
                                "collection.eno" => {
                                    hyper_dir.collection_manifest = Some(path.join("collection.eno"));
                                }
                                "playlist.eno" => {
                                    hyper_dir.playlist_manifest = Some(path.join("playlist.eno"));
                                }
                                "site.eno" => {
                                    hyper_dir.site_manifest = Some(path.join("site.eno"));
                                }
                                "video.eno" => {
                                    hyper_dir.video_manifest = Some(path.join("video.eno"));
                                }
                                file_name => {
                                    match Path::new(file_name).extension() {
                                        Some(extension) => {
                                            // TODO: Further refactor to directly check against
                                            // extension everywhere instead of passing and
                                            // disassembling filenames again and again?
                                            if VideoFile::supported_extension(file_name) {
                                                hyper_dir.video_files.push(dir_entry.path());
                                            } else if supported_image_extension(file_name) {
                                                hyper_dir.image_files.push(dir_entry.path());
                                            } else if AudioFile::supported_extension(file_name) {
                                                hyper_dir.audio_files.push(dir_entry.path());
                                            } else if let Ok(format) = SubtitleFormat::try_from(extension) {
                                                let path = dir_entry.path();

                                                let subtitle_file = HyperSubtitleFile {
                                                    format,
                                                    path
                                                };

                                                hyper_dir.subtitle_files.push(subtitle_file);
                                            } else if supported_font_extension(file_name) {
                                                hyper_dir.font_files.push(dir_entry.path());
                                            } else if extension.eq_ignore_ascii_case("eno") {
                                                let error = format!("Ignoring unknown manifest type {file_name} (known manifests are collection.eno, playlist.eno, site.eno, video.eno)");
                                                hyper_dir.errors.push(error);
                                            } else {
                                                let error = format!("Ignoring unsupported file {file_name} (unsupported extension {extension:?})");
                                                hyper_dir.errors.push(error);
                                            }
                                        }
                                        None => {
                                            let error = format!("Ignoring file {file_name} without extension");
                                            hyper_dir.errors.push(error);
                                        }
                                    }
                                }
                            }
                        } else if file_type.is_symlink() {
                            let error = format!("Ignoring symlink '{}'", dir_entry.path().display());
                            hyper_dir.errors.push(error);
                        } else {
                            let error = format!("Ignoring file '{}' of unsupported type", dir_entry.path().display());
                            hyper_dir.errors.push(error);
                        }
                    }
                }
            }
        }

        hyper_dir
    }

    /// HyperDir stores a platform-specific, absolute path internally. With
    /// this method it instead returns the relative [SitePath] inside the
    /// site directory.
    pub fn site_path(&self, context: &Context) -> SitePath {
        SitePath::from_path(context, &self.path)
    }
}

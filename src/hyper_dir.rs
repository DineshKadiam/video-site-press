use std::path::{Path, PathBuf};

use slug::slugify;

use crate::Context;
use crate::util::{
    supported_audio_extension,
    supported_font_extension,
    supported_image_extension,
    supported_subtitle_extension,
    supported_video_extension
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
    pub subtitle_files: Vec<PathBuf>,
    pub video_files: Vec<PathBuf>,
    pub video_manifest: Option<PathBuf>
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
                        if file_type.is_dir() {
                            let file_name = dir_entry.file_name().to_string_lossy().to_string();

                            if file_name.starts_with('.') { continue }

                            let slugified = slugify(&file_name);

                            // TODO: Revisit whether slugify(...) actually imposes the sort of dir name constraints
                            //       we want, and also, what are these anyway, exactly?
                            if slugified == file_name {
                                hyper_dir.dirs.push(dir_entry.path());
                            } else {
                                let error = format!("Ignoring directory '{file_name}' because its name is invalid (try to only use alphanumeric characters, dash and underscore, e.g. '{slugified}')");
                                hyper_dir.errors.push(error);
                            }
                        } else if file_type.is_file() {
                            match dir_entry.file_name().to_string_lossy().as_ref() {
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
                                filename => {
                                    if supported_video_extension(filename) {
                                        hyper_dir.video_files.push(dir_entry.path());
                                    } else if supported_image_extension(filename) {
                                        hyper_dir.image_files.push(dir_entry.path());
                                    } else if supported_audio_extension(filename) {
                                        hyper_dir.audio_files.push(dir_entry.path());
                                    } else if supported_subtitle_extension(filename) {
                                        hyper_dir.subtitle_files.push(dir_entry.path());
                                    } else if supported_font_extension(filename) {
                                        hyper_dir.font_files.push(dir_entry.path());
                                    } else if filename.ends_with(".eno") {
                                        let error = format!("Ignoring unknown manifest type {} (known manifests are collection.eno, playlist.eno, site.eno, video.eno)", filename);
                                        hyper_dir.errors.push(error);
                                    } else {
                                        let error = format!("Ignoring unsupported file {} (unsupported extension)", filename);
                                        hyper_dir.errors.push(error);
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

    pub fn site_path(&self, context: &Context) -> String {
        self.path
            .strip_prefix(&context.site_dir)
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
    }
}

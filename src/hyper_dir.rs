use std::path::{Path, PathBuf};

use slug::slugify;

use crate::Context;

/// Convenience wrapper to generically pre-scan directories for
/// repeating/required patterns used in hyper8's folder hierarchy.
#[derive(Debug)]
pub struct HyperDir {
    pub audio_files: Vec<PathBuf>,
    pub collection_manifest: Option<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub image_files: Vec<PathBuf>,
    pub invalid_dirs: Vec<String>,
    pub path: PathBuf,
    pub playlist_manifest: Option<PathBuf>,
    pub site_manifest: Option<PathBuf>,
    pub video_files: Vec<PathBuf>,
    pub video_manifest: Option<PathBuf>
}

impl HyperDir {
    fn new(path: &Path) -> HyperDir {
        HyperDir {
            audio_files: Vec::new(),
            collection_manifest: None,
            dirs: Vec::new(),
            image_files: Vec::new(),
            invalid_dirs: Vec::new(),
            path: path.to_owned(),
            playlist_manifest: None,
            site_manifest: None,
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

                            // TODO: Revisit whether slugify(...) actually imposes the sort of dir name constraints
                            //       we want, and also, what are these anyway, exactly?
                            if slugify(&file_name) == file_name {
                                hyper_dir.dirs.push(dir_entry.path());
                            } else {
                                hyper_dir.invalid_dirs.push(file_name);
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
                                    let path = dir_entry.path();
                                    if let Some(extension) = path.extension() {
                                        match extension.to_string_lossy().to_lowercase().as_str() {
                                            "aac" | "aiff" | "alac" | "flac" | "mp3" | "ogg" | "opus" | "wav" => {
                                                hyper_dir.audio_files.push(dir_entry.path());
                                            }
                                            "eno" => {
                                                eprintln!("Ignoring unknown manifest type {} (known manifests are collection.eno, playlist.eno, site.eno, video.eno)", filename);
                                            }
                                            "gif" | "jpeg" | "jpg" | "png" | "webp" => {
                                                hyper_dir.image_files.push(dir_entry.path());
                                            }
                                            "m4v" | "mp4" | "mkv" | "mov" => {
                                                hyper_dir.video_files.push(dir_entry.path());
                                            }
                                            unsupported_extension => {
                                                eprintln!("Ignoring unsupported file {} (.{} is an unsupported extension)", path.display(), unsupported_extension);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if file_type.is_symlink() {
                            eprintln!("Ignoring symlink '{}'", dir_entry.path().display());
                        } else {
                            eprintln!("Ignoring file '{}' of unsupported type", dir_entry.path().display());
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

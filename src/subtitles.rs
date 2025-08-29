// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};

use language_tags::LanguageTag;
use rsubs_lib::{SRT, VTT};

#[derive(Clone, Debug)]
pub struct Subtitle {
    pub start: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct SubtitleFile {
    pub explicitly_defined: bool,
    pub file_name: String,
    pub label: String,
    /// Specified as BCP 47 code (e.g. 'en')
    /// (see e.g. https://www.rfc-editor.org/info/bcp47)
    pub language: Option<String>,
    // Err() represents that the content of the subtitles file
    // could not be parsed by us (including why).
    pub subtitles: Result<Vec<Subtitle>, String>
}

impl Subtitle {
    pub fn new(start: String, text: String) -> Subtitle {
        Subtitle {
            start,
            text
        }
    }
}

impl SubtitleFile {
    /// Ensure the language is already validated when passing it here.
    pub fn from_manifest(
        label_option: Option<String>,
        language: Option<String>,
        path: PathBuf
    ) -> SubtitleFile {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let label = label_option.unwrap_or_else(||
            path.file_stem().unwrap().to_string_lossy().to_string()
        );

        let subtitles = SubtitleFile::parse_subtitles(&file_name, &path);

        SubtitleFile {
            explicitly_defined: true,
            file_name,
            label,
            language,
            subtitles
        }
    }

    pub fn from_path(path: &Path) -> SubtitleFile {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

        // If the file stem is a valid BCP 47 language code we automatically derive the language from it
        let language = match LanguageTag::parse(&file_stem).is_ok_and(|language_tag| language_tag.is_valid()) {
            true => Some(file_stem.clone()),
            false => None
        };

        let subtitles = SubtitleFile::parse_subtitles(&file_name, &path);

        SubtitleFile {
            explicitly_defined: false,
            file_name,
            label: file_stem,
            language,
            subtitles
        }
    }

    pub fn parse_subtitles(file_name: &str, path: &Path) -> Result<Vec<Subtitle>, String> {
        let content = fs::read_to_string(path).unwrap();

        if file_name.ends_with(".vtt") {
            match VTT::parse(content) {
                Ok(vtt) => {
                    let subtitles = vtt
                        .lines
                        .iter()
                        .map(|vtt_line| {
                            let hour = vtt_line.start.hour();
                            let minute = vtt_line.start.minute();
                            let second = vtt_line.start.second();

                            let start = if hour > 0 {
                                format!("{hour:02}:{minute:02}:{second:02}")
                            } else {
                                format!("{minute:02}:{second:02}")
                            };

                            let text = vtt_line.text.clone();

                            Subtitle::new(start, text)
                        })
                        .collect::<Vec<Subtitle>>();

                    Ok(subtitles)
                }
                Err(err) => Err(err.to_string())
            }
        } else if file_name.ends_with(".srt") {
            match SRT::parse(content) {
                Ok(srt) => {
                    let subtitles = srt
                        .lines
                        .iter()
                        .map(|srt_line| {
                            let hour = srt_line.start.hour();
                            let minute = srt_line.start.minute();
                            let second = srt_line.start.second();

                            let start = if hour > 0 {
                                format!("{hour:02}:{minute:02}:{second:02}")
                            } else {
                                format!("{minute:02}:{second:02}")
                            };

                            let text = srt_line.text.clone();

                            Subtitle::new(start, text)
                        })
                        .collect::<Vec<Subtitle>>();

                    Ok(subtitles)
                }
                Err(err) => Err(err.to_string())
            }
        } else {
            unreachable!()
        }
    }
}

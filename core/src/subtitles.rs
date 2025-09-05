// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};

use language_tags::LanguageTag;
use rsubs_lib::{SRT, VTT};

/// Represents a single "line" in a subtitles file
#[derive(Clone, Debug)]
pub struct Subtitle {
    pub start: Timecode,
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

#[derive(Clone, Debug)]
pub struct Timecode {
    pub hour: u8,
    pub millisecond: u16,
    pub minute: u8,
    pub second: u8
}

impl Subtitle {
    pub fn new(start: Timecode, text: String) -> Subtitle {
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
                            let start = Timecode::new(
                                vtt_line.start.hour(),
                                vtt_line.start.millisecond(),
                                vtt_line.start.minute(),
                                vtt_line.start.second()
                            );

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
                            let start = Timecode::new(
                                srt_line.start.hour(),
                                srt_line.start.millisecond(),
                                srt_line.start.minute(),
                                srt_line.start.second()
                            );

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

impl Timecode {
    // Output a display version of the timecode, such as for
    // instance "13:02", "00:23" or "1:33:29". The milliseconds
    // are always omitted for this.
    pub fn format_display(&self) -> String {
        let hour = self.hour;
        let minute = self.minute;
        let second = self.second;

        if hour > 0 {
            format!("{hour}:{minute:02}:{second:02}")
        } else {
            format!("{minute:02}:{second:02}")
        }
    }

    // Outputs a hash-parameter based link to the timecode like for instance
    // "#time=1h13s", "#time=14m313ms", or "#time=0s". This includes milliseconds,
    // because they can make a noticeable difference when jumping around in
    // subtitled narration.
    pub fn format_link(&self) -> String {
        let mut timecode = String::new();

        let hour = self.hour;
        let minute = self.minute;
        let millisecond = self.millisecond;
        let second = self.second;

        if hour > 0 {
            timecode.push_str(&format!("{hour}h"));
        }

        if minute > 0 {
            timecode.push_str(&format!("{minute}m"));
        }

        if second > 0 {
            timecode.push_str(&format!("{second}s"));
        }

        if millisecond > 0 {
            timecode.push_str(&format!("{millisecond}ms"));
        }

        if timecode.is_empty() {
            timecode.push_str("0s");
        }

        format!("#time={timecode}")
    }

    pub fn new(
        hour: u8,
        millisecond: u16,
        minute: u8,
        second: u8
    ) -> Timecode {
        Timecode {
            hour,
            millisecond,
            minute,
            second
        }
    }
}

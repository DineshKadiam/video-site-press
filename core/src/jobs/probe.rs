// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::mem;
use std::path::Path;
use std::process::{Command};

use serde_derive::Deserialize;

use hyper8_translations::Translations;

use crate::{AudioMeta, VideoMeta};

pub const FFPROBE: &str = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct FFprobeResult {
    pub format: FFprobeFormat,
    pub streams: Vec<FFprobeStream>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct FFprobeFormat {
    pub duration: String, // float
    pub format_long_name: String,
    pub format_name: String,
    pub nb_streams: usize,
    pub size: String // int
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct FFprobeStream {
    pub avg_frame_rate: String, // e.g. "24/1"
    /// Once observed to be missing for a stream of codec_type "data" with the
    /// codec_tag_string "tmcd" and codec_tag "0x64636d74".
    pub codec_long_name: Option<String>,
    /// Once observed to be missing for a stream of codec_type "data" with the
    /// codec_tag_string "tmcd" and codec_tag "0x64636d74".
    pub codec_name: Option<String>,
    /// This can be "audio", "data", "video"
    pub codec_type: String,
    pub duration: Option<String>, // float
    pub height: Option<usize>,
    pub nb_frames: Option<String>, // int
    pub pix_fmt: Option<String>,
    pub sample_rate: Option<String>, // int
    pub side_data_list: Option<Vec<FFprobeStreamSideData>>,
    pub width: Option<usize>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct FFprobeStreamSideData {
    pub displaymatrix: Option<String>,
    pub rotation: Option<i16>,
    /// We are currently only interested in "Display Matrix"
    pub side_data_type: String
}
fn parse_frame_rate(representation: &str) -> f32 {
    let mut components = representation.split('/');

    let dividend = components.next().unwrap().parse::<f32>().unwrap();
    let divisor = components.next().unwrap().parse::<f32>().unwrap();

    dividend / divisor
}

pub fn probe_audio(
    path: &Path,
    translations: &Translations
) -> Result<AudioMeta, String> {
    let mut command = Command::new(FFPROBE);

    command
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-show_format")
        .arg(path);

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                match serde_json::from_slice::<FFprobeResult>(&output.stdout) {
                    Ok(result) => {
                        let mut codec = None;
                        let mut duration = 0.0;

                        if let Ok(format_duration) = &result.format.duration.parse::<f32>() {
                            duration = *format_duration;
                        }

                        for stream in &result.streams {
                            if let Some(stream_duration) = &stream.duration {
                                if let Ok(stream_duration) = stream_duration.parse::<f32>() {
                                    if stream_duration > duration {
                                        duration = stream_duration;
                                    }
                                }
                            }

                            match stream.codec_type.as_str() {
                                "audio" => {
                                    // For audio streams we assume there is always a codec name
                                    codec = Some(stream.codec_name.as_ref().unwrap().clone());
                                }
                                "data" => {
                                    // Was once observed, (probably) containing timecode data of some sorts.
                                    // We don't do anything with this.
                                }
                                "video" => {
                                    // We don't do anything with this.
                                }
                                // For now crash in this case, just to be aware if there
                                // are other things that we might want to leverage here.
                                // (subtitles? embedded thumbnail frames? etc.)
                                other => unimplemented!("'{other}' stream not implemented")
                            }
                        }

                        let audio_meta = AudioMeta::new(codec, duration);

                        Ok(audio_meta)
                    }
                    Err(err) => panic!("{err}")
                }
            } else {
                let message = format!("{output:?}");
                Err(message)
            }
        }
        Err(err) => {
            let the_ffprobe_child_process_could_not_be_executed = translations.the_xxx_child_process_could_not_be_executed("ffprobe");
            let message = format!("{the_ffprobe_child_process_could_not_be_executed} ({err})");
            Err(message)
        }
    }
}

pub fn probe_video(
    path: &Path,
    translations: &Translations
) -> Result<VideoMeta, String> {
    let mut command = Command::new(FFPROBE);

    command
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-show_format")
        .arg(path);

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                match serde_json::from_slice::<FFprobeResult>(&output.stdout) {
                    Ok(result) => {
                        let mut audio_codec = None;
                        let mut duration = 0.0;
                        let mut height = None;
                        let mut video_codec = None;
                        let mut video_frame_rate = 0.0;
                        let mut width = None;

                        if let Ok(format_duration) = &result.format.duration.parse::<f32>() {
                            duration = *format_duration;
                        }

                        for stream in &result.streams {
                            if let Some(stream_duration) = &stream.duration {
                                if let Ok(stream_duration) = stream_duration.parse::<f32>() {
                                    if stream_duration > duration {
                                        duration = stream_duration;
                                    }
                                }
                            }

                            // TODO: Technically there could be multiple audio/video codecs?
                            //       Not sure how this would be handled or how this would look.
                            match stream.codec_type.as_str() {
                                "audio" => {
                                    // For audio streams we assume there is always a codec name
                                    audio_codec = Some(stream.codec_name.as_ref().unwrap().clone());
                                }
                                "data" => {
                                    // Was once observed, (probably) containing timecode data of some sorts.
                                    // We don't do anything with this.
                                }
                                "video" => {
                                    // For video streams we assume there is always a codec name
                                    video_codec = Some(stream.codec_name.as_ref().unwrap().clone());
                                    video_frame_rate = parse_frame_rate(&stream.avg_frame_rate);

                                    if height.is_none() { height = stream.height; }
                                    if width.is_none() { width = stream.width; }

                                    // If there is side data that indicates the video needs to be
                                    // rotated for display, we flip width and height in our metadata
                                    if stream.side_data_list
                                        .as_ref()
                                        .is_some_and(|side_data_list| {
                                            side_data_list
                                                .iter()
                                                .any(|side_data| {
                                                    side_data.side_data_type == "Display Matrix" &&
                                                    side_data.rotation.is_some_and(|rotation|
                                                        rotation.abs() % 180 == 90
                                                    )
                                                })
                                        }) {
                                        mem::swap(&mut height, &mut width);
                                    }
                                }
                                // For now crash in this case, just to be aware if there
                                // are other things that we might want to leverage here.
                                // (subtitles? embedded thumbnail frames? etc.)
                                other => unimplemented!("'{other}' stream not implemented")
                            }
                        }

                        let video_meta = VideoMeta::new(
                            audio_codec,
                            duration,
                            height.unwrap(),
                            video_codec,
                            video_frame_rate,
                            width.unwrap()
                        );

                        Ok(video_meta)
                    }
                    Err(err) => panic!("{err}")
                }
            } else {
                let message = format!("{output:?}");
                Err(message)
            }
        }
        Err(err) => {
            let the_ffprobe_child_process_could_not_be_executed = translations.the_xxx_child_process_could_not_be_executed("ffprobe");
            let message = format!("{the_ffprobe_child_process_could_not_be_executed} ({err})");
            Err(message)
        }
    }
}

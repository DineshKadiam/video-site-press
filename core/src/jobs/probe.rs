// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::mem;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_derive::Deserialize;

use crate::{AudioMeta, VideoMeta};
use crate::util::collect_output_stderr_stdout;

use super::JobError;

fn ffprobe_executable() -> PathBuf {
    // If the HYPER8_BUNDLED_FFMPEG environment variable is specified at build
    // time, ffprobe is expected to be a binary adjacent to the hyper8 binary
    // itself. We use this for building and packaging on platforms that
    // require bundled versions of ffmpeg to be shipped with Hyper 8.
    if option_env!("HYPER8_BUNDLED_FFMPEG").is_some() {
        env::current_exe().unwrap().parent().unwrap().join("ffprobe")
    } else {
        PathBuf::from("ffprobe")
    }
}

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

pub fn probe_audio(path: &Path) -> Result<AudioMeta, JobError> {
    let mut command = Command::new(ffprobe_executable());

    command
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-show_format")
        .arg(path);

    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            let job_error = JobError::ChildProcessCouldNotBeExecuted {
                err: err.to_string(),
                program: "ffprobe"
            };
            return Err(job_error);
        }
    };

    if !output.status.success() {
        return Err(JobError::Generic { err: collect_output_stderr_stdout(output) });
    }

    let ffprobe_result = match serde_json::from_slice::<FFprobeResult>(&output.stdout) {
        Ok(ffprobe_result) => ffprobe_result,
        Err(err) => {
            let err = format!("Encountered ffprobe output that is not yet handled by Hyper 8 - please report this as a bug, including this message: '{err}'");
            return Err(JobError::Generic { err });
        }
    };

    let mut codec = None;
    let mut duration = 0.0;

    if let Ok(format_duration) = &ffprobe_result.format.duration.parse::<f32>() {
        duration = *format_duration;
    }

    for stream in &ffprobe_result.streams {
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
            other_type => {
                // Intentionally crash so we are made aware that there
                // are other things that we might want to leverage here.
                // (subtitles? embedded thumbnail frames? etc.)
                let err = format!("Encountered ffprobe stream.codec_type that is not yet handled by Hyper 8 - please report this as a bug, including this type information: '{other_type}'");
                return Err(JobError::Generic { err })
            }
        }
    }

    let audio_meta = AudioMeta::new(codec, duration);

    Ok(audio_meta)
}

pub fn probe_video(path: &Path) -> Result<VideoMeta, JobError> {
    let mut command = Command::new(ffprobe_executable());

    command
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-show_format")
        .arg(path);

    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            let job_error = JobError::ChildProcessCouldNotBeExecuted {
                err: err.to_string(),
                program: "ffprobe"
            };
            return Err(job_error);
        }
    };

    if !output.status.success() {
        return Err(JobError::Generic { err: collect_output_stderr_stdout(output) });
    }

    let ffprobe_result = match serde_json::from_slice::<FFprobeResult>(&output.stdout) {
        Ok(ffprobe_result) => ffprobe_result,
        Err(err) => {
            let err = format!("Encountered ffprobe output that is not yet handled by Hyper 8 - please report this as a bug, including this message: '{err}'");
            return Err(JobError::Generic { err });
        }
    };

    let mut audio_codec = None;
    let mut duration = 0.0;
    let mut height = None;
    let mut video_codec = None;
    let mut video_frame_rate = 0.0;
    let mut width = None;

    if let Ok(format_duration) = &ffprobe_result.format.duration.parse::<f32>() {
        duration = *format_duration;
    }

    for stream in &ffprobe_result.streams {
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
            other_type => {
                // Intentionally crash so we are made aware that there
                // are other things that we might want to leverage here.
                // (subtitles? embedded thumbnail frames? etc.)
                let err = format!("Encountered ffprobe stream.codec_type that is not yet handled by Hyper 8 - please report this as a bug, including this type information: '{other_type}'");
                return Err(JobError::Generic { err });
            }
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

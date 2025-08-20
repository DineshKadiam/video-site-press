// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use serde_derive::Deserialize;

use crate::{
    AudioFile,
    AudioFormat,
    AudioMeta,
    Codec,
    Context,
    EncodingApproach,
    FileMeta,
    JobProgress,
    Translations,
    Video,
    VideoFile,
    VideoFormat,
    VideoMeta
};
use crate::util::{deduplicate_filename, nanoid};

pub const FFMPEG: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
pub const FFPROBE: &str = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };

// We could also (but currently don't) use the following progress fields that
// ffmpeg reports: fps, q, size, bitrate, speed
// These can easily be added in FFmpegEncodingStats::parse if needed.
#[allow(dead_code)]
pub struct FFmpegEncodingStats {
    /// How many frames of the video were encoded so far
    pub frame: usize,
    /// Up to which timecode in the video encoding has progressed so far
    pub time: f32
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

/// With `timecode` (in seconds) we specify at which point in the video we
/// take the snapshot for the poster.
pub fn compute_poster(context: &Context, timecode: f32, video: &Video) -> Result<String, ()> {
    if let Some(video_file) = &video.highest_quality_video_file() {
        let mut command = Command::new(FFMPEG);

        let input_path = context.site_dir
            .join(video.site_path.filesystem_relative())
            .join(&video_file.file_name);

        let mut output_filename = String::from("poster.jpg");
        let mut output_path = context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&output_filename);

        if output_path.exists() {
            output_filename = format!("poster_{}.jpg", timecode);
            output_path = context.site_dir
                .join(video.site_path.filesystem_relative())
                .join(&output_filename);

            while output_path.exists() {
                output_filename = deduplicate_filename(&output_filename);
                output_path = context.site_dir
                    .join(video.site_path.filesystem_relative())
                    .join(&output_filename);
            }
        }

        command
            .arg("-y")
            .arg("-ss")
            .arg(timecode.to_string())
            .arg("-i")
            .arg(&input_path)
            .arg("-frames")
            .arg("1")
            .arg("-q:v")
            .arg("7") // 1-31 (best-worst)
            .arg(&output_path);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(output_filename);
                }

                eprintln!("{output:?}");
                Err(())
            }
            Err(err) => {
                eprintln!("The ffmpeg child process could not be executed.");
                eprintln!("{err}");
                Err(())
            }
        }
    } else {
        eprintln!("There was no source video to compute a poster from. ({})", video.site_path.normalized());
        Err(())
    }
}

/// Based on the passed [Video], kick off an ffmpeg-based transcoding process
/// to encode the video as an audio-only file. During processing this calls
/// methods on context to update the progress for the job (using the passed
/// job_id).
pub fn encode_audio(
    audio_format: AudioFormat,
    context: &Arc<Context>,
    job_id: usize,
    translations: &Translations,
    video: &Video
) -> Result<AudioFile, String> {
    let mut command = Command::new(FFMPEG);

    // Prevent ffmpeg from prompting about overwriting files
    command.arg("-y");

    // Disable interaction on stdin from ffmpeg's side
    command.arg("-nostdin");

    // Suppress all output but explicitly re-enable periodic printing
    // of progress (which we parse and process further).
    command
        .arg("-v").arg("quiet")
        .arg("-stats")
        .arg("-stats_period").arg("1");

    let highest_quality_video_file = match video.highest_quality_video_file() {
        Some(video_file) => video_file,
        None => return Err(String::from("No video file found to encode audio from"))
    };

    let input_path = context.site_dir
        .join(video.site_path.filesystem_relative())
        .join(&highest_quality_video_file.file_name);

    command.arg("-i").arg(&input_path);

    // Disable any video output
    command.arg("-vn");

    match audio_format {
        AudioFormat::Mp3 => {
            command.arg("-codec:a").arg("libmp3lame");
            command.arg("-qscale:a").arg("0");
        }
        AudioFormat::Opus => {
            command.arg("-codec:a").arg("libopus");
            command.arg("-b:a").arg("128k");
        }
    }

    let extension = audio_format.extension();

    let temp_file = format!("{}.{extension}", nanoid());
    let temp_path = context.cache_dir.join(&temp_file);

    command.arg(&temp_path);

    // We're reading from stderr to receive progress updates from the ffmpeg background process.
    command.stderr(Stdio::piped());

    // Adressing the theoretical possibility that open but unattended stdxxx handles
    // could lead to to e.g. memory buffer overflows or other forms of the ffmpeg
    // background process going stale, we're here disabling these unneeded streams.
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());

    let rendering_begin = Instant::now();

    match command.spawn() {
        Ok(mut child) => {
            if let Some(video_meta) = &highest_quality_video_file.video_meta {
                let video_duration = video_meta.duration;

                match child.stderr.take() {
                    Some(stderr) => {
                        let context_moving = Arc::clone(context);
                        thread::spawn(move || {
                            let stderr_reader = BufReader::with_capacity(1024, stderr);

                            for result in stderr_reader.split(b'\r') {
                                if let Ok(bytes) = result {
                                    if let Ok(line) = std::str::from_utf8(&bytes) {
                                        let duration = rendering_begin.elapsed().as_secs_f32();
                                        let stats = FFmpegEncodingStats::parse(line);
                                        let job_progress = JobProgress::new(duration, stats, video_duration);
                                        context_moving.update_job_progress(job_id, job_progress);
                                    }
                                }
                            }
                        });
                    }
                    None => {
                        let message = String::from("Could not capture stderr for ffmpeg child process");
                        return Err(message)
                    }
                }
            }

            let exit_status = match child.wait() {
                Ok(exit_status) => exit_status,
                Err(_) => {
                    let message = String::from("The ffmpeg child process could not be waited for");
                    return Err(message)
                }
            };

            if exit_status.success() {
                // If the video directory disappeared (was moved/removed) while we were encoding
                // we discard our encoding output as we don't know where to put it anymore.
                if !context.site_dir.join(video.site_path.filesystem_relative()).exists() {
                    let message = String::from("The video directory has disappeared while we encoded audio, the encoded audio was discarded");
                    return Err(message)
                }

                let file_stem = input_path.file_stem().unwrap().to_string_lossy().to_string();

                let mut iteration = 0;
                let (output_file, output_path) = loop {
                    let output_file = if iteration == 0 {
                        format!("{file_stem}.{extension}")
                    } else {
                        format!("{file_stem}({iteration}).{extension}")
                    };

                    let output_path = context.site_dir
                        .join(video.site_path.filesystem_relative())
                        .join(&output_file);

                    if output_path.exists() {
                        iteration += 1;
                        continue;
                    }

                    break (output_file, output_path);
                };

                // Move file from cache dir to video dir
                fs::rename(&temp_path, &output_path).unwrap();

                let file_meta = FileMeta::new(&output_path);
                let audio_file = AudioFile::new(None, file_meta, output_file);

                Ok(audio_file)
            } else {
                let error_code = match exit_status.code() {
                    Some(code) => format!("error code {code}"),
                    None => String::from("probably signal-terminated")
                };

                let the_ffmpeg_child_process_terminated_unsuccessfully = translations.the_xxx_child_process_terminated_unsuccessfully("ffmpeg");
                let message = format!("{the_ffmpeg_child_process_terminated_unsuccessfully} ({error_code})");
                Err(message)
            }
        }
        Err(err) => {
            let the_ffmpeg_child_process_could_not_be_spawned = translations.the_xxx_child_process_could_not_be_spawned("ffmpeg");
            let message = format!("{the_ffmpeg_child_process_could_not_be_spawned} ({err})");
            Err(message)
        }
    }
}

pub fn encode_video(
    context: &Arc<Context>,
    job_id: usize,
    translations: &Translations,
    video: &Video,
    video_format: VideoFormat
) -> Result<VideoFile, String> {
    let mut command = Command::new(FFMPEG);

    // Prevent ffmpeg from prompting about overwriting files
    command.arg("-y");

    // Disable interaction on stdin from ffmpeg's side
    command.arg("-nostdin");

    // Suppress all output but explicitly re-enable periodic printing
    // of progress (which we parse and process further).
    command
        .arg("-v").arg("quiet")
        .arg("-stats")
        .arg("-stats_period").arg("1");

    let highest_quality_video_file = match video.highest_quality_video_file() {
        Some(video_file) => video_file,
        None => return Err(String::from("No video file found to encode audio from"))
    };

    let input_path = context.site_dir
        .join(video.site_path.filesystem_relative())
        .join(&highest_quality_video_file.file_name);

    command.arg("-i").arg(&input_path);

    let mut video_filters = Vec::new();

    if let Some(frame_rate) = video_format.frame_rate {
        video_filters.push(format!("fps={frame_rate}"));
    }

    // TODO: Omit this argument if we don't do any resizing - either by checking against
    //       original resolution here, or by implementing a "keep original" VideoFormat
    //       enum-like indication.
    let height = video_format.height;
    video_filters.push(format!("scale=-2:{height}"));

    command.arg("-vf").arg(video_filters.join(","));

    // TODO: Right now the best supported video format is H.264, but VP9 is now also supported across
    //       all browsers (since March 2024), so as soon as old Safari versions leave
    //       the playing field we can transition to VP9 as recommended default.
    //       See https://caniuse.com/webm
    //
    //       HEVC as default is hardly an outlook given its support is very sparse and has been
    //       for a long time - https://caniuse.com/hevc - and probably not going to change anytime
    //       soon (also because it is not royalty free).
    //
    //       Support for AV1 is already quite decent - https://caniuse.com/av1 - although
    //       not omnipresent due to Safari's unreliable support, but this is possibly a long-term
    //       perspective for a future default.

    match video_format.codec {
        Codec::H264 => {
            // See https://trac.ffmpeg.org/wiki/Encode/H.264
            command.arg("-c:v").arg("libx264");

            match video_format.encoding_approach {
                EncodingApproach::AverageResults => command.arg("-preset").arg("medium"),
                EncodingApproach::HighestCompression => command.arg("-preset").arg("veryslow"),
                EncodingApproach::HighestSpeed => command.arg("-preset").arg("ultrafast")
            };

            // "The range of the CRF scale is 0–51, where 0 is lossless (for 8 bit only,
            // for 10 bit use -qp 0), 23 is the default, and 51 is worst quality
            // possible. A lower value generally leads to higher quality, and a
            // subjectively sane range is 17–28. Consider 17 or 18 to be visually
            // lossless or nearly so; it should look the same or nearly the same as the
            // input but it isn't technically lossless."
            //
            // Our quality goes from 0-1 (worst-best), we map to 18-51 (best-worst)
            let crf = 18 + ((1.0 - video_format.quality) * 33.0) as u8;
            command.arg("-crf").arg(crf.to_string());

            command.arg("-movflags").arg("+faststart");
        }
        Codec::Hevc => {
            // See https://trac.ffmpeg.org/wiki/Encode/H.265
            command.arg("-c:v").arg("libx265");

            match video_format.encoding_approach {
                EncodingApproach::AverageResults => command.arg("-preset").arg("medium"),
                EncodingApproach::HighestCompression => command.arg("-preset").arg("veryslow"),
                EncodingApproach::HighestSpeed => command.arg("-preset").arg("ultrafast")
            };

            // "Choose a CRF. CRF affects the quality. The default is
            // 28, and it should visually correspond to libx264 video
            // at CRF 23, but result in about half the file size. CRF
            // works just like in x264, so choose the highest value
            // that provides an acceptable quality."
            //
            // Our quality goes from 0-1 (worst-best), we map to 18-51 (best-worst)
            let crf = 18 + ((1.0 - video_format.quality) * 33.0) as u8;
            command.arg("-crf").arg(crf.to_string());

            command.arg("-tag:v").arg("hvc1");
        }
        Codec::Vp9 => {
            // TODO: Actually two-pass encoding yields best results for VP9,
            //       but this will require some architectural changes.
            // See https://trac.ffmpeg.org/wiki/Encode/VP9
            // See https://developers.google.com/media/vp9/settings/vod/
            command.arg("-c:v").arg("libvpx-vp9");

            match video_format.encoding_approach {
                EncodingApproach::AverageResults => command.arg("-deadline").arg("good"),
                EncodingApproach::HighestCompression => command.arg("-deadline").arg("best"),
                EncodingApproach::HighestSpeed => command.arg("-deadline").arg("realtime")
            };

            // "The CRF value can be from 0–63. Lower values mean
            // better quality. Recommended values range from 15–35,
            // with 31 being recommended for 1080p HD video."
            //
            // Our quality goes from 0-1 (worst-best), we map to 15-35 (best-worst)
            let crf = 15 + ((1.0 - video_format.quality) * 20.0) as u8;
            command.arg("-crf").arg(crf.to_string());
            command.arg("-b:v").arg("0");
        }
    }

    let extension = video_format.container.extension();
    let temp_file = format!("{}.{extension}", nanoid());
    let temp_path = context.cache_dir.join(&temp_file);

    command.arg(&temp_path);

    // We're reading from stderr to receive progress updates from the ffmpeg background process.
    command.stderr(Stdio::piped());

    // Adressing the theoretical possibility that open but unattended stdxxx handles
    // could lead to to e.g. memory buffer overflows or other forms of the ffmpeg
    // background process going stale, we're here disabling these unneeded streams.
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());

    let rendering_begin = Instant::now();

    match command.spawn() {
        Ok(mut child) => {
            if let Some(video_meta) = &highest_quality_video_file.video_meta {
                let video_duration = video_meta.duration;

                // TODO: This is NOT a reliable figure to measure encoded frames against to compute progress
                // in percent, reason being that ffmpeg may noticably alter the target frame rate
                // depending on the used format and encoder during encoding (and we would only know afterwards,
                // unless we maybe enable and parse the additional up-front info from the ffmpeg invocation)
                // let video_frames_estimated = match &format {
                //     VideoFormat::Video { frame_rate: Some(target_frame_rate), .. } => {
                //         (target_frame_rate * video_meta.duration) as usize
                //     }
                //     _ => (video_meta.video_frame_rate * video_meta.duration) as usize
                // };

                match child.stderr.take() {
                    Some(stderr) => {
                        let context_moving = Arc::clone(context);
                        thread::spawn(move || {
                            let stderr_reader = BufReader::with_capacity(1024, stderr);

                            for result in stderr_reader.split(b'\r') {
                                if let Ok(bytes) = result {
                                    if let Ok(line) = std::str::from_utf8(&bytes) {
                                        let duration = rendering_begin.elapsed().as_secs_f32();
                                        let stats = FFmpegEncodingStats::parse(line);
                                        let job_progress = JobProgress::new(duration, stats, video_duration);
                                        context_moving.update_job_progress(job_id, job_progress);
                                    }
                                }
                            }
                        });
                    }
                    None => {
                        let message = String::from("Could not capture stderr for ffmpeg child process");
                        return Err(message)
                    }
                }
            }

            let exit_status = match child.wait() {
                Ok(exit_status) => exit_status,
                Err(_) => {
                    let message = String::from("The ffmpeg child process could not be waited for");
                    return Err(message)
                }
            };

            if exit_status.success() {
                // If the video directory disappeared (was moved/removed) while we were encoding
                // we discard our encoding output as we don't know where to put it anymore.
                if !context.site_dir.join(video.site_path.filesystem_relative()).exists() {
                    let message = String::from("The video directory has disappeared while we encoded video, the encoded audio was discarded");
                    return Err(message)
                }

                let file_stem = input_path.file_stem().unwrap().to_string_lossy().to_string();
                let format_suffix = video_format.file_name_suffix();
                let base_name = format!("{file_stem}_{format_suffix}");

                let mut iteration = 0;
                let (output_file, output_path) = loop {
                    let output_file = if iteration == 0 {
                        format!("{base_name}.{extension}")
                    } else {
                        format!("{base_name}({iteration}).{extension}")
                    };

                    let output_path = context.site_dir
                        .join(video.site_path.filesystem_relative())
                        .join(&output_file);

                    if output_path.exists() {
                        iteration += 1;
                        continue;
                    }

                    break (output_file, output_path);
                };

                // Move file from cache dir to video dir
                fs::rename(&temp_path, &output_path).unwrap();

                let file_meta = FileMeta::new(&output_path);
                let video_file = VideoFile::new(file_meta, output_file, None);

                Ok(video_file)
            } else {
                let error_code = match exit_status.code() {
                    Some(code) => format!("error code {code}"),
                    None => String::from("probably signal-terminated")
                };

                let the_ffmpeg_child_process_terminated_unsuccessfully = translations.the_xxx_child_process_terminated_unsuccessfully("ffmpeg");
                let message = format!("{the_ffmpeg_child_process_terminated_unsuccessfully} ({error_code})");
                Err(message)
            }
        }
        Err(err) => {
            let the_ffmpeg_child_process_could_not_be_spawned = translations.the_xxx_child_process_could_not_be_spawned("ffmpeg");
            let message = format!("{the_ffmpeg_child_process_could_not_be_spawned} ({err})");
            Err(message)
        }
    }
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
                                        let height_memo = height;
                                        height = width;
                                        width = height_memo;
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

impl FFmpegEncodingStats {
    pub fn parse(line: &str) -> FFmpegEncodingStats {
        let mut frame = 0;
        let mut time = 0.0;

        let mut parse_token = |key: &str, value: &str| {
            match key {
                "frame" => {
                    if let Ok(number) = value.parse::<usize>() {
                        frame = number;
                    }
                }
                "time" => {
                    // We observed ffmpeg returning negative timecodes on rare occasions,
                    // e.g. in this log sequence (showing full `line` values here):
                    //
                    // frame=    1 fps=0.0 q=0.0 size=       0kB time=00:00:00.00 bitrate=N/A speed=   0x"
                    // frame=   95 fps= 87 q=29.0 size=       0kB time=-00:00:00.04 bitrate=N/A speed=N/A"
                    // frame=  145 fps= 67 q=29.0 size=     256kB time=00:00:02.04 bitrate=1027.3kbits/s speed=0.941x"
                    // (...)
                    //
                    if !value.starts_with('-') {
                        // hh:mm:ss.ff (e.g. 00:01:06.54)
                        let hours = value[0..2].parse::<usize>().unwrap();
                        let minutes = value[3..5].parse::<usize>().unwrap();
                        let seconds = value[6..11].parse::<f32>().unwrap();
                        time = (hours * 3600) as f32 + (minutes * 60) as f32 + seconds;
                    }
                }
                _ => ()
            }
        };

        let mut remainder = line.trim();

        loop {
            if let Some((key, padding_value_next_remainder)) = remainder.split_once('=') {
                let value_next_remainder = padding_value_next_remainder.trim_start();
                if let Some((value, next_remainder)) = value_next_remainder.split_once(' ') {
                    parse_token(key, value);
                    remainder = next_remainder;
                } else {
                    parse_token(key, value_next_remainder);
                    break
                }
            }
        }

        FFmpegEncodingStats {
            frame,
            time
        }
    }
}

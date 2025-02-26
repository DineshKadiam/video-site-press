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
    Codec,
    Context,
    EncodingApproach,
    FileMeta,
    Progress,
    Video,
    VideoFile,
    VideoFormat,
    VideoMeta
};
use crate::util::nanoid;

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
    pub width: Option<usize>
}

/// With seek_offset (0.0-1.0) we determine at which point in the video (start to end)
/// we take the snapshot for the poster
pub fn compute_poster(context: &Context, seek_offset: f32, video: &Video) -> Result<String, ()> {
    if let Some(version) = &video.highest_quality_version() {
        let mut command = Command::new(FFMPEG);

        let input_path = context.site_dir.join(&video.path).join(&version.file_name);

        // If we know the duration (which conceptually speaking we should be guaranteed to know),
        // we take the poster snapshot at halftime, otherwise at the start
        let snapshot_seek_offset_seconds = version.video_meta
            .as_ref()
            .map(|video_meta| format!("{}", video_meta.duration * seek_offset))
            .unwrap_or("0".to_string());

        let output_filename = if !context.site_dir.join(&video.path).join("poster.jpg").exists() {
            "poster.jpg".to_string()
        } else {
            let timecoded = format!("poster_{}.jpg", snapshot_seek_offset_seconds);

            if !context.site_dir.join(&video.path).join(&timecoded).exists() {
                timecoded
            } else {
                format!("poster_{}.jpg", nanoid())
            }
        };

        let output_path = context.site_dir.join(&video.path).join(&output_filename);

        command
            .arg("-y")
            .arg("-ss")
            .arg(snapshot_seek_offset_seconds)
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

                eprint!("{output:?}");
                Err(())
            }
            Err(err) => {
                eprint!("The ffmpeg child process could not be executed.");
                eprint!("{err}");
                Err(())
            }
        }
    } else {
        eprint!("There was no source video to compute a poster from. ({})", &video.path);
        Err(())
    }
}

pub fn encode(
    context: &Arc<Context>,
    format: VideoFormat,
    process_id: usize,
    video: &Video
) -> Result<VideoFile, ()> {
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

    let highest_quality_version = match video.highest_quality_version() {
        Some(version) => version,
        None => return Err(())
    };

    let input_path = context.site_dir.join(&video.path).join(&highest_quality_version.file_name);

    command.arg("-i").arg(&input_path);

    let extension = match &format {
        VideoFormat::AudioOnly => {
            command
                .arg("-vn") // no video
                .arg("-codec:a").arg("libopus")
                .arg("-b:a").arg("96k");

            // TODO: We rather want mp3+opus (?)
            // TODO: mp4? Is there a merit to using a video-less mp4 over a pure audio format?
            "mp4"
        }
        VideoFormat::Video { codec, container, encoding_approach, frame_rate, height, quality } => {
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

            let mut video_filters = Vec::new();

            if let Some(frame_rate) = frame_rate {
                video_filters.push(format!("fps={frame_rate}"));
            }

            // TODO: Omit this argument if we don't do any resizing - either by checking against
            //       original resolution here, or by implementing a "keep original" VideoFormat
            //       enum-like indication.
            video_filters.push(format!("scale=-2:{height}"));

            command.arg("-vf").arg(video_filters.join(","));

            match codec {
                Codec::H264 => {
                    // See https://trac.ffmpeg.org/wiki/Encode/H.264
                    command.arg("-c:v").arg("libx264");

                    match encoding_approach {
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
                    let crf = 18 + ((1.0 - quality) * 33.0) as u8;
                    command.arg("-crf").arg(crf.to_string());

                    command.arg("-movflags").arg("+faststart");
                }
                Codec::Hevc => {
                    // See https://trac.ffmpeg.org/wiki/Encode/H.265
                    command.arg("-c:v").arg("libx265");

                    match encoding_approach {
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
                    let crf = 18 + ((1.0 - quality) * 33.0) as u8;
                    command.arg("-crf").arg(crf.to_string());

                    command.arg("-tag:v").arg("hvc1");
                }
                Codec::Vp9 => {
                    // TODO: Actually two-pass encoding yields best results for VP9,
                    //       but this will require some architectural changes.
                    // See https://trac.ffmpeg.org/wiki/Encode/VP9
                    // See https://developers.google.com/media/vp9/settings/vod/
                    command.arg("-c:v").arg("libvpx-vp9");

                    match encoding_approach {
                        EncodingApproach::AverageResults => command.arg("-deadline").arg("good"),
                        EncodingApproach::HighestCompression => command.arg("-deadline").arg("best"),
                        EncodingApproach::HighestSpeed => command.arg("-deadline").arg("realtime")
                    };

                    // "The CRF value can be from 0–63. Lower values mean
                    // better quality. Recommended values range from 15–35,
                    // with 31 being recommended for 1080p HD video."
                    //
                    // Our quality goes from 0-1 (worst-best), we map to 15-35 (best-worst)
                    let crf = 15 + ((1.0 - quality) * 20.0) as u8;
                    command.arg("-crf").arg(crf.to_string());
                    command.arg("-b:v").arg("0");
                }
            }

            container.extension()
        }
    };

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
            if let Some(video_meta) = &highest_quality_version.video_meta {
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
                                        let progress = Progress::new(duration, stats, video_duration);
                                        context_moving.processing.lock().unwrap().update_progress(process_id, progress);
                                    }
                                }
                            }
                        });
                    }
                    None => {
                        eprint!("Could not capture stderr for ffmpeg child process.");
                        return Err(())
                    }
                }
            }

            let exit_status = match child.wait() {
                Ok(exit_status) => exit_status,
                Err(_) => {
                    eprint!("The ffmpeg child process could not be waited for.");
                    return Err(())
                }
            };

            if exit_status.success() {
                // If the video directory disappeared (was moved/removed) while we were encoding
                // we discard our encoding output as we don't know where to put it anymore.
                if !context.site_dir.join(&video.path).exists() {
                    return Err(())
                }

                let file_stem = input_path.file_stem().unwrap().to_string_lossy().to_string();

                let base_name = match &format {
                    VideoFormat::AudioOnly => {
                        format!("{file_stem}_audio-only")
                    }
                    VideoFormat::Video { codec, encoding_approach, frame_rate, height, quality, .. } => {
                        let approach_suffix = encoding_approach.to_string();
                        let codec_suffix = codec.to_string();
                        let frame_rate_suffix = match frame_rate {
                            Some(frame_rate) => format!("_{frame_rate}fps"),
                            None => String::new()
                        };
                        let quality_suffix = format!("{:.0}", quality * 100.0);

                        format!("{file_stem}_{height}p{frame_rate_suffix}_{codec_suffix}_q{quality_suffix}_{approach_suffix}")
                    }
                };

                let mut iteration = 0;
                let (output_file, output_path) = loop {
                    let output_file = if iteration == 0 {
                        format!("{base_name}.{extension}")
                    } else {
                        format!("{base_name}({iteration}).{extension}")
                    };

                    let output_path = context.site_dir.join(&video.path).join(&output_file);

                    if output_path.exists() {
                        iteration += 1;
                        continue;
                    }

                    break (output_file, output_path);
                };

                // Move file from cache dir to video dir
                fs::rename(&temp_path, &output_path).unwrap();

                let file_meta = FileMeta::new(&output_path);
                
                let video_version = VideoFile {
                    file_meta,
                    file_name: output_file,
                    video_meta: None
                };

                Ok(video_version)
            } else {
                let error_code = match exit_status.code() {
                    Some(code) => format!("error code {code}"),
                    None => String::from("probably signal-terminated")
                };
                eprint!("The ffmpeg child process terminated unsuccessfully ({error_code})");
                Err(())
            }
        }
        Err(err) => {
            eprint!("The ffmpeg child process could not be spawned.");
            eprint!("{err}");
            Err(())
        }
    }
}

pub fn ffprobe(path: &Path) -> Result<VideoMeta, ()> {
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
                                }
                                // For now crash in this case, just to be aware if there
                                // are other things that we might want to leverage here.
                                // (subtitles? embedded thumbnail frames? etc.)
                                other => unimplemented!("'{other}' stream not implemented")
                            }
                        }

                        let metadata = VideoMeta::new(
                            audio_codec,
                            duration,
                            height.unwrap(),
                            video_codec,
                            video_frame_rate,
                            width.unwrap()
                        );

                        return Ok(metadata)
                    }
                    Err(err) => panic!("{}", err)
                }
            }

            eprint!("{output:?}");
            Err(())
        }
        Err(err) => {
            eprint!("The ffmpeg child process could not be executed.");
            eprint!("{err}");
            Err(())
        }
    }
}

fn parse_frame_rate(representation: &str) -> f32 {
    let mut components = representation.split('/');

    let dividend = components.next().unwrap().parse::<f32>().unwrap();
    let divisor = components.next().unwrap().parse::<f32>().unwrap();

    dividend / divisor
}

impl FFmpegEncodingStats {
    pub fn parse(line: &str) -> FFmpegEncodingStats {
        let mut frame = 0;
        let mut time = 0.0;

        let mut parse_token = |key: &str, value: &str| {
            match key {
                "frame" => if let Ok(number) = value.parse::<usize>() {
                    frame = number;
                }
                "time" => {
                    // hh:mm:ss.ff (e.g. 00:01:06.54)
                    let hours = value[0..2].parse::<usize>().unwrap();
                    let minutes = value[3..5].parse::<usize>().unwrap();
                    let seconds = value[6..11].parse::<f32>().unwrap();
                    time = (hours * 3600) as f32 + (minutes * 60) as f32 + seconds;
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

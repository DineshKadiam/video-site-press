use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde_derive::Deserialize;

use crate::{
    Codec,
    Context,
    EncodingApproach,
    FileMeta,
    Video,
    VideoFile,
    VideoFormat,
    VideoMeta
};
use crate::util::nanoid;

pub const FFMPEG: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
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
    pub codec_long_name: String,
    pub codec_name: String,
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
            .map(|video_meta| format!("{}", &video_meta.duration * seek_offset))
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
            .arg("-vf")
            .arg("scale=min'(480,iw)':-2")
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
    context: &Context,
    format: VideoFormat,
    video: &Video
) -> Result<VideoFile, ()> {
    let mut command = Command::new(FFMPEG);

    // Prevent ffmpeg from prompting about overwriting files
    command.arg("-y");

    let input_path = if let Some(version) = video.highest_quality_version() {
        context.site_dir.join(&video.path).join(&version.file_name)
    } else {
        return Err(());
    };

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
        VideoFormat::Video { codec, container, encoding_approach, height, quality } => {
            // TODO: We need to encode WebM and h265 for browser compatibility reasons
            //       Consider that we also need to/could offer both for download
            // See also https://pixelpoint.io/blog/web-optimized-video-ffmpeg/

            command.arg("-vf").arg(format!("scale=-2:{height}"));

            match codec {
                Codec::H264 => {
                    // See https://trac.ffmpeg.org/wiki/Encode/H.264
                    command.arg("-c:v").arg("libx264");

                    match encoding_approach {
                        EncodingApproach::Efficiency => command.arg("-preset").arg("veryslow"),
                        EncodingApproach::Speed => command.arg("-preset").arg("ultrafast")
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
                        EncodingApproach::Efficiency => command.arg("-preset").arg("veryslow"),
                        EncodingApproach::Speed => command.arg("-preset").arg("ultrafast")
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
                        EncodingApproach::Efficiency => command.arg("-deadline").arg("best"),
                        EncodingApproach::Speed => command.arg("-deadline").arg("realtime")
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

    let rendering_begin = Instant::now();

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                let _rendering_time = rendering_begin.elapsed().as_secs_f32();

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
                    VideoFormat::Video { codec, encoding_approach, height, quality, .. } => {
                        let approach_suffix = encoding_approach.to_string();
                        let codec_suffix = codec.to_string();
                        let quality_suffix = format!("{:.0}", quality * 100.0);

                        format!("{file_stem}_{height}p_{codec_suffix}_q{quality_suffix}_{approach_suffix}")
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

            	return Ok(video_version);
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
                                    audio_codec = Some(stream.codec_name.clone());
                                }
                                "video" => {
                                    video_codec = Some(stream.codec_name.clone());

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
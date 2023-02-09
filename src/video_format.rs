use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;

// TODO: Research state of AV1 encoding (ffmpeg etc.), future outlook for support
//       https://en.wikipedia.org/wiki/AV1
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Codec {
    H264,
    Hevc,
    Vp9
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Container {
    Mkv,
    Mp4
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncodingApproach {
    Quality,
    Speed
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VideoFormat {
    AudioOnly,
    Video {
        codec: Codec,
        container: Container,
        encoding_approach: EncodingApproach,
        // TODO: Can we somehow type-enforce a range? Brittle right now.
        quality: f32, // 0.0 (worst) to 1.0 (best)
        width: usize
    }
}

impl Codec {
    pub fn to_string(&self) -> &str {
        match self {
            Codec::H264 => "h264",
            Codec::Hevc => "hevc",
            Codec::Vp9 => "vp9"
        }
    }
}

impl Container {
    pub fn to_string(&self) -> &str {
        match self {
            Container::Mkv => "mkv",
            Container::Mp4 => "mp4"
        }
    }
}

impl EncodingApproach {
    pub fn to_string(&self) -> &str {
        match self {
            EncodingApproach::Quality => "quality",
            EncodingApproach::Speed => "speed"
        }
    }
}

impl VideoFormat {
    /// Determine the priority-based ordering between this and another format.
    /// Video is prioritized over audio, higher resolution over lower resolution.
    pub fn cmp_priority(&self, other: &VideoFormat) -> Ordering {
        match self {
            VideoFormat::AudioOnly => match other {
                VideoFormat::Video { .. } => Ordering::Less,
                VideoFormat::AudioOnly => Ordering::Equal
            }
            VideoFormat::Video { width, .. } => match other {
                VideoFormat::AudioOnly => Ordering::Greater,
                VideoFormat::Video { width: other_width, .. } => width.cmp(other_width)
            }
        }
    }

    pub fn encode(&self) -> String {
        match self {
            VideoFormat::AudioOnly => String::from("audio-only"),
            VideoFormat::Video { codec, container, encoding_approach, quality, width } => {
                format!(
                    "video:{}:{}:{}:{quality}:{width}",
                    codec.to_string(),
                    container.to_string(),
                    encoding_approach.to_string()
                )
            }
        }
    }

    // TODO: Should we output encoding approach, quality etc. (probably yes but check contexts of usage)
    pub fn label(&self) -> String {
        match self {
            VideoFormat::AudioOnly => "audio-only".to_string(),
            VideoFormat::Video { width, .. } => width.to_string()
        }
    }

    pub fn parse(string: &str) -> Option<VideoFormat> {
        let mut components = string.split(':');

        match components.next() {
            Some("audio-only") => {
                Some(VideoFormat::AudioOnly)
            }
            Some("video") => {
                let codec = match components.next() {
                    Some("h264") => Codec::H264,
                    Some("hevc") => Codec::Hevc,
                    Some("vp9") => Codec::Vp9,
                    _ => return None
                };

                let container = match components.next() {
                    Some("mkv") => Container::Mkv,
                    Some("mp4") => Container::Mp4,
                    _ => return None
                };

                let encoding_approach = match components.next() {
                    Some("quality") => EncodingApproach::Quality,
                    Some("speed") => EncodingApproach::Speed,
                    _ => return None
                };

                let quality = match components.next() {
                    Some(quality_str) => match quality_str.parse::<f32>() {
                        Ok(quality) => quality,
                        Err(_) => return None
                    }
                    None => return None
                };

                let width = match components.next() {
                    Some(width_str) => match width_str.parse::<usize>() {
                        Ok(width) => width,
                        Err(_) => return None
                    }
                    None => return None
                };

                Some(VideoFormat::Video {
                    codec,
                    container,
                    encoding_approach,
                    quality,
                    width
                })
            }
            _ => None
        }
    }
}

impl PartialEq for VideoFormat {
    /// We currently ignore encoding_approach and quality to determine
    /// equality. There is a trade-off to make between having all material in
    /// exactly the formats the user wants, and causing lots and lots of lengthy
    /// video recomputation ... The idea is currently that the user can always
    /// request a re-encoding if they want better quality on a video.
    fn eq(&self, other: &VideoFormat) -> bool {
        match self {
            VideoFormat::AudioOnly => {
                if let VideoFormat::AudioOnly = other { return true; }
            }
            VideoFormat::Video { width, .. } => {
                if let VideoFormat::Video { width: other_width, .. } = &other {
                    if width == other_width { return true; }
                }
            }
        }

        false
    }
}

impl Eq for VideoFormat {}

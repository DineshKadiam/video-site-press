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
    Efficiency,
    Speed
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VideoFormat {
    AudioOnly,
    Video {
        codec: Codec,
        container: Container,
        encoding_approach: EncodingApproach,
        height: usize,
        // TODO: Can we somehow type-enforce a range? Brittle right now.
        quality: f32 // 0.0 (worst) to 1.0 (best)
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
    pub fn extension(&self) -> &str {
        match self {
            Container::Mkv => "mkv",
            Container::Mp4 => "mp4"
        }
    }
}

impl EncodingApproach {
    pub fn to_string(&self) -> &str {
        match self {
            EncodingApproach::Efficiency => "efficiency",
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
            VideoFormat::Video { height, .. } => match other {
                VideoFormat::AudioOnly => Ordering::Greater,
                VideoFormat::Video { height: other_height, .. } => height.cmp(other_height)
            }
        }
    }

    pub fn encode(&self) -> String {
        match self {
            VideoFormat::AudioOnly => String::from("audio-only"),
            VideoFormat::Video { codec, container, encoding_approach, height, quality } => {
                format!(
                    "video:{}:{}:{}:{quality}:{height}",
                    codec.to_string(),
                    container.extension(),
                    encoding_approach.to_string()
                )
            }
        }
    }

    // TODO: Should we output encoding approach, quality etc. (probably yes but check contexts of usage)
    pub fn label(&self) -> String {
        match self {
            VideoFormat::AudioOnly => "audio-only".to_string(),
            VideoFormat::Video { height, .. } => format!("{height}p")
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
                    Some("efficiency") => EncodingApproach::Efficiency,
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

                let height = match components.next() {
                    Some(height_str) => match height_str.parse::<usize>() {
                        Ok(height) => height,
                        Err(_) => return None
                    }
                    None => return None
                };

                Some(VideoFormat::Video {
                    codec,
                    container,
                    encoding_approach,
                    height,
                    quality
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
            VideoFormat::Video { height, .. } => {
                if let VideoFormat::Video { height: other_height, .. } = &other {
                    if height == other_height { return true; }
                }
            }
        }

        false
    }
}

impl Eq for VideoFormat {}

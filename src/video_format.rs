use std::cmp::Ordering;

// TODO: Research state of AV1 encoding (ffmpeg etc.), future outlook for support
//       https://en.wikipedia.org/wiki/AV1
#[derive(Clone, Debug, PartialEq)]
pub enum Codec {
    H264,
    Hevc,
    Vp9
}

#[derive(Clone, Debug, PartialEq)]
pub enum Container {
    Mkv,
    Mp4
}

#[derive(Clone, Debug, PartialEq)]
pub enum EncodingApproach {
    AverageResults,
    HighestCompression,
    HighestSpeed
}

#[derive(Clone, Debug, PartialEq)]
pub enum VideoFormat {
    AudioOnly,
    Video {
        codec: Codec,
        container: Container,
        encoding_approach: EncodingApproach,
        /// Represents fps
        frame_rate: Option<f32>,
        height: usize,
        // TODO: Can we somehow type-enforce a range? Brittle right now.
        /// 0.0 (worst) to 1.0 (best)
        quality: f32
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
            EncodingApproach::AverageResults => "average",
            EncodingApproach::HighestCompression => "compression",
            EncodingApproach::HighestSpeed => "speed"
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

    // TODO: Should we output encoding approach, quality etc. (probably yes but check contexts of usage)
    pub fn label(&self) -> String {
        match self {
            VideoFormat::AudioOnly => "audio-only".to_string(),
            VideoFormat::Video { height, .. } => format!("{height}p")
        }
    }

    pub fn new_video(
        codec: &str,
        container: &str,
        encoding_approach: &str,
        frame_rate: Option<&str>,
        height: &str,
        quality: &str,
    ) -> Option<VideoFormat> {
        let codec = match codec {
           "h264" => Codec::H264,
           "hevc" => Codec::Hevc,
           "vp9" => Codec::Vp9,
            _ => return None
        };

        let container = match container {
           "mkv" => Container::Mkv,
           "mp4" => Container::Mp4,
            _ => return None
        };

        let encoding_approach = match encoding_approach {
           "average" => EncodingApproach::AverageResults,
           "compression" => EncodingApproach::HighestCompression,
           "speed" => EncodingApproach::HighestSpeed,
            _ => return None
        };

        let frame_rate = match frame_rate {
            Some(frame_rate) => match frame_rate.parse::<f32>() {
                Ok(frame_rate) => Some(frame_rate),
                Err(_) => return None
            }
            None => None
        };

        let quality = match quality.parse::<f32>() {
            Ok(quality) => quality,
            Err(_) => return None
        };

        let height = match height.parse::<usize>() {
            Ok(height) => height,
            Err(_) => return None
        };

        Some(VideoFormat::Video {
            codec,
            container,
            encoding_approach,
            frame_rate,
            height,
            quality
        })
    }
}

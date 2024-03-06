use serde_derive::{Deserialize, Serialize};

// TODO: This is all nice, but generally it would be better if we store all the factors
//       we need to determine browser support in the (also cached) metadata, so that future updates of
//       hyper8 either improve browser support determination based on the already existing
//       metadata, or alternatively, that cached metadata is deleted and refreshed, because additional
//       markers are considered then, so this also improves browser support determination and does
//       trigger the necessary updates (unlike in the current case where determination is frozen forever!)
//
//       TODO: Also: We might need to consider some things in combination (e.g., say, codec_name + pix_fmt together)

/// A best-effort determination of whether a video is playable in
/// a standard browser, based on its metadata (from ffprobe).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrowserSupport {
    supported: Vec<String>,
    unsupported: Vec<String>
}

impl BrowserSupport {
    /// Compatible (Ok) and Incompatible (Err) include a message that describes
    /// why the format was deemed (in)compatible.
    pub fn analysis(&self) -> Option<Result<String, String>> {
        if !self.unsupported.is_empty() {
            Some(Err(self.unsupported.join(", ")))
        } else if !self.supported.is_empty() {
            Some(Ok(self.supported.join(", ")))
        } else {
            None
        }
    }

    pub fn consider_audio_codec_name(&mut self, codec_name: &str) {
        if codec_name == "aac" {
            self.supported.push("AAC audio codec is generally supported in browsers".to_string());
        }
    }

    pub fn consider_video_codec_name(&mut self, codec_name: &str) {
        if codec_name == "h264" {
            self.supported.push("H264 video codec is generally supported in browsers".to_string());
        }
    }

    pub fn new() -> BrowserSupport {
        BrowserSupport {
            supported: Vec::new(),
            unsupported: Vec::new()
        }
    }
}
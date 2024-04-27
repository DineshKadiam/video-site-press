use crate::VideoMeta;

/// Compatible (Ok) and Incompatible (Err) include a message that describes
/// why the format was deemed (in)compatible.
pub fn analysis(video_meta: &VideoMeta) -> Option<Result<String, String>> {
    let mut supported: Vec<String> = Vec::new();
    let mut unsupported: Vec<String> = Vec::new();

    if video_meta.audio_codec.as_ref().is_some_and(|codec| codec == "aac") {
        supported.push("AAC audio codec is generally supported in browsers".to_string());
    }

    if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "h264") {
        supported.push("H264 video codec is generally supported in browsers".to_string());
    } else if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "vp9") {
        supported.push("VP9 video codec is generally supported in browsers".to_string());
    }

    if !unsupported.is_empty() {
        Some(Err(unsupported.join(", ")))
    } else if !supported.is_empty() {
        Some(Ok(supported.join(", ")))
    } else {
        None
    }
}
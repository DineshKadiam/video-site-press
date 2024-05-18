use crate::{Language, VideoMeta};

/// Compatible (Ok) and Incompatible (Err) include a message that describes
/// why the format was deemed (in)compatible.
pub fn analysis(language: &Language, video_meta: &VideoMeta) -> Option<Result<String, String>> {
    let mut supported: Vec<String> = Vec::new();
    let unsupported: Vec<String> = Vec::new();

    let t_generally_supported_in_browsers = language.translations.generally_supported_in_browsers;

    if video_meta.audio_codec.as_ref().is_some_and(|codec| codec == "aac") {
        let message = format!("AAC audio codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    }

    if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "h264") {
        let message = format!("H264 video codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    } else if video_meta.video_codec.as_ref().is_some_and(|codec| codec == "vp9") {
        let message = format!("VP9 video codec: {t_generally_supported_in_browsers}");
        supported.push(message);
    }

    if !unsupported.is_empty() {
        Some(Err(unsupported.join(", ")))
    } else if !supported.is_empty() {
        Some(Ok(supported.join(", ")))
    } else {
        None
    }
}
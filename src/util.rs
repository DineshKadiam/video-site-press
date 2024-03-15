use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::ErrorKind;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use nanoid::nanoid;
use slug::slugify;
use url::Url;

pub struct Field {
    pub error: Option<String>,
    pub value: String
}

impl Field {
    pub fn valid<V>(value: V) -> Field
        where V: Into<String>
    {
        Field {
            error: None,
            value: value.into()
        }
    }

    pub fn validated<E, V>(value: V, error: Option<E>) -> Field
        where E: Into<String>, V: Into<String>
    {
        Field {
            error: error.map(|message| message.into()),
            value: value.into()
        }
    }
}

/// Takes seconds, formats it like "25 min", "33 sec", "158 min"
pub fn casual_duration(seconds: f32) -> String {
    if seconds < 60.0 {
        format!("{} sec", seconds as u8)
    } else {
        format!("{} min", (seconds / 60.0).round() as u32)
    }
}

pub fn ensure_dir(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
}

pub fn ensure_empty_dir(dir: &Path) {
    remove_dir(dir);
    fs::create_dir_all(dir).unwrap();
}

/// To avoid using excessive amounts of space and performing unneccessary
/// copying operations we for some files create hard links instead of copies.
/// If hard links can not be created (e.g. because cache and build directory
/// are on different file systems) we just silently fall back to regular
/// copying.
pub fn hard_link_or_copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) {
    fs::hard_link(&from, &to)
        .unwrap_or_else(|_| {
            fs::copy(&from, &to).unwrap();
        });
}

pub fn hash(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    URL_SAFE_NO_PAD.encode(hasher.finish().to_le_bytes())
}

/// Takes seconds, formats it like "0:13", "3:15", "2:06:48"
pub fn precise_duration(seconds: f32) -> String {
    if seconds < 60.0 {
        format!("0:{:02}", seconds as u16)
    } else if seconds < 3600.0 {
        format!(
            "{}:{:02}",
            seconds as u16 / 60,
            seconds as u16 % 60
        )
    } else {
        format!(
            "{}:{:02}:{:02}",
            seconds as u16 / 3600,
            (seconds as u16 % 3600) / 60,
            seconds as u16 % 60
        )
    }
}

pub fn human_size(bytes: u64) -> String {
    match bytes {
        0..=999 => format!("{bytes}B"),
        1000..=999999 => format!("{}kB", bytes / 1000),
        1000000..=999999999 => format!("{:.1}MB", bytes as f64 / 1000000.0),
        _ => format!("{:.2}GB", bytes as f64 / 1000000000.0),
    }
}

pub fn nanoid() -> String {
    nanoid!(8)
}

pub fn remove_dir(dir: &Path) {
    match fs::remove_dir_all(dir) {
        Ok(()) => (),
        Err(ref err) if err.kind() == ErrorKind::NotFound => (), // just what we want anyway \o/
        Err(err) => panic!("{}", err)
    };
}

pub fn remove_file(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => (),
        Err(ref err) if err.kind() == ErrorKind::NotFound => (), // just what we want anyway \o/
        Err(err) => panic!("{}", err)
    }
}

pub fn supported_audio_extension(filename: &str) -> bool {
    match Path::new(filename).extension() {
        Some(extension) => {
            extension.eq_ignore_ascii_case("aac") ||
            extension.eq_ignore_ascii_case("aiff") ||
            extension.eq_ignore_ascii_case("alac") ||
            extension.eq_ignore_ascii_case("flac") ||
            extension.eq_ignore_ascii_case("mp3") ||
            extension.eq_ignore_ascii_case("ogg") ||
            extension.eq_ignore_ascii_case("opus") ||
            extension.eq_ignore_ascii_case("wav")
        }
        None => false
    }
}

pub fn supported_image_extension(filename: &str) -> bool {
    match Path::new(filename).extension() {
        Some(extension) => {
            extension.eq_ignore_ascii_case("gif") ||
            extension.eq_ignore_ascii_case("jpg") ||
            extension.eq_ignore_ascii_case("jpeg") ||
            extension.eq_ignore_ascii_case("png") ||
            extension.eq_ignore_ascii_case("webp")
        }
        None => false
    }
}

pub fn supported_video_extension(filename: &str) -> bool {
    match Path::new(filename).extension() {
        Some(extension) => {
            extension.eq_ignore_ascii_case("m4v") ||
            extension.eq_ignore_ascii_case("mp4") ||
            extension.eq_ignore_ascii_case("mkv") ||
            extension.eq_ignore_ascii_case("mov")
        }
        None => false
    }
}

pub fn timestamp() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

/// If the slug comes from user input make sure to trim it before passing it.
/// This checks whether the slug is non-empty and whether it contains only
/// allowed characters.
pub fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err(String::from("Can't be empty"));
    }

    let slugified = slugify(slug);
    if slugified != slug {
        return Err(format!("Not a slug, allowed is e.g. {slugified}"));
    }

    Ok(())
}

/// If the url comes from user input make sure to trim it before passing it.
pub fn validate_url(url: &str) -> Result<(), String> {
    Url::parse(url)
        .map(|_url| ())
        .map_err(|err| err.to_string())
}

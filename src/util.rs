use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Locale as ChronoLocale, NaiveDate};
use nanoid::nanoid;
use slug::slugify;

use crate::{Context, Translations};

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

/// Takes seconds, formats it like "25 min", "33 sec", "158 min", "3.6 hrs", "11 hrs"
pub fn casual_duration(seconds: f32, translations: &Translations) -> String {
    if seconds < 60.0 {
        let t_sec = translations.sec;
        format!("{} {t_sec}", seconds as u8)
    } else if seconds < 240.0 * 60.0 {
        let t_min = translations.min;
        format!("{} {t_min}", (seconds / 60.0).round() as u32)
    } else {
        let hours = seconds / (10.0 * 60.0 * 60.0);

        let t_hrs = translations.hrs;
        if hours >= 10.0 || hours.fract() < 0.1 {
            format!("{} {t_hrs}", hours.round() as u32)
        } else {
            format!("{hours:.1} {t_hrs}")
        }
    }
}

pub fn checked_remove_dir_all(context: &Context, path: &Path) -> Result<(), Error> {
    if !path.starts_with(&context.site_dir) {
        panic!(
            "[CRITICAL, PLEASE READ] Aborted recursive directory removal of '{}', which was determined to be outside of the site directory '{}'.\nThis is a highly critical error and should be reported to https://codeberg.org/simonrepp/hyper8/issues (alongside a short description after which action the error occurred).\nYour bug report will crucially help to avoid data loss in the future, thank you!",
            path.display(),
            context.site_dir.display()
        );
    }

    fs::remove_dir_all(path)
}

pub fn ensure_dir(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
}

pub fn ensure_empty_dir(dir: &Path) {
    match fs::remove_dir_all(dir) {
        Ok(()) => (),
        Err(ref err) if err.kind() == ErrorKind::NotFound => (), // just what we want anyway \o/
        Err(err) => panic!("{}", err)
    };
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

pub fn hash_closure(closure: impl FnOnce(&mut DefaultHasher)) -> String {
    let mut hasher = DefaultHasher::new();
    closure(&mut hasher);
    URL_SAFE_NO_PAD.encode(hasher.finish().to_le_bytes())
}

/// Escape e.g. "love>hate" so it can be rendered into the page,
/// e.g. as <span>love&gt;hate</span>
pub fn html_escape_outside_attribute(string: &str) -> String {
    string.replace('&', "&amp;")
          .replace('<', "&lt;")
          .replace('>', "&gt;")
}

/// Takes bytes, formats them like "313 B", "256 kB", "3.4 MB", "13 MB", "1.4 GB"
pub fn human_size(bytes: u64) -> String {
    match bytes {
        0..=999 => format!("{bytes} B"),
        1000..=999999 => {
            let kilobytes = bytes / 1000;
            format!("{kilobytes} kB")
        }
        1000000..=999999999 => {
            let megabytes = bytes as f64 / 1000000.0;
            if megabytes >= 10.0 || megabytes.fract() < 0.1 {
                format!("{} MB", megabytes as u64)
            } else {
                format!("{megabytes:.1} MB")
            }
        },
        _ => {
            let gigabytes = bytes as f64 / 1000000000.0;
            if gigabytes.fract() < 0.1 {
                format!("{} GB", gigabytes as u64)
            } else if gigabytes.fract() < 0.01 {
                format!("{gigabytes:.1} GB")
            } else {
                format!("{gigabytes:.2} GB")
            }
        }
    }
}

pub fn localized_date_d_mmmm_yyyy(date: NaiveDate, chrono_locale: ChronoLocale) -> String {
    use chrono::format::{Fixed, Item, Numeric, Pad};

    let format = [
        Item::Numeric(Numeric::Day, Pad::None),
        Item::Space(" "),
        Item::Fixed(Fixed::LongMonthName),
        Item::Space(" "),
        Item::Numeric(Numeric::Year, Pad::None)
    ];

    date
        .format_localized_with_items(format.iter(), chrono_locale)
        .to_string()
}

pub fn localized_date_mmm_yyyy(date: NaiveDate, chrono_locale: ChronoLocale) -> String {
    use chrono::format::{Fixed, Item, Numeric, Pad};

    let format = [
        Item::Fixed(Fixed::ShortMonthName),
        Item::Space(" "),
        Item::Numeric(Numeric::Year, Pad::None)
    ];

    date
        .format_localized_with_items(format.iter(), chrono_locale)
        .to_string()
}

pub fn nanoid() -> String {
    nanoid!(8)
}

/// Takes seconds, formats it like "00:13", "03:15", "2:06:48"
pub fn precise_duration(seconds: f32) -> String {
    if seconds < 60.0 {
        format!("00:{:02}", seconds as u16)
    } else if seconds < 3600.0 {
        format!(
            "{:02}:{:02}",
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

pub fn supported_font_extension(filename: &str) -> bool {
    match Path::new(filename).extension() {
        Some(extension) => {
            extension.eq_ignore_ascii_case("woff") ||
            extension.eq_ignore_ascii_case("woff2")
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

pub fn supported_subtitle_extension(filename: &str) -> bool {
    match Path::new(filename).extension() {
        Some(extension) => {
            extension.eq_ignore_ascii_case("srt") ||
            extension.eq_ignore_ascii_case("vtt")
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
            extension.eq_ignore_ascii_case("mov") ||
            extension.eq_ignore_ascii_case("webm")
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

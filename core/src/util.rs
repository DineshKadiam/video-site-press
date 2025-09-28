// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::{OsStr, OsString};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Output;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Locale as ChronoLocale, NaiveDate};
use nanoid::nanoid;
use slug::slugify;

use hyper8_translations::Translations;

use crate::Context;

/// When we perform updates on collections, playlists and videos in the site
/// tree, the underlying mutation is often infallible, therefore the error
/// branch related to the update itself is unreachable in these cases. This
/// macro serves as a reusable marker to indicate these circumstances.
#[macro_export]
macro_rules! infallible_mutation {
    () => { unreachable!("Infallible mutation") };
}

pub type InfallibleResult = Result<(), ()>;

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

pub fn collect_output_stderr_stdout(output: Output) -> String {
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    format!("stderr: {}\n\nstdout: {}", stderr, stdout)
}

/// Takes an existing filestem (that has elsewhere been identified to collide
/// with another) and returns the filestem, systematically altered as to
/// resolve the existing collision and ideally avoid further ones.
///
/// Note: For proper file*names* (with extension) use deduplicate_filename instead!
///
/// Some examples:
/// - "foo"            -> "foo(1)"
/// - "foo(1)"         -> "foo(2)"
pub fn deduplicate_filestem(filestem: &str) -> String {
    if filestem.ends_with(')') {
        if let Some(index) = filestem.rfind('(') {
            if let Ok(number) = &filestem[(index + 1)..(filestem.len() - 1)].parse::<usize>() {
                return format!("{}({})", &filestem[..index], number + 1);
            }
        }
    }

    format!("{filestem}(1)")
}

/// Takes an existing filename (that has elsewhere been identified to collide
/// with another file with the same filename) and returns the filename,
/// systematically altered as to resolve the existing collision and ideally
/// avoid further ones.
///
/// Some examples:
/// - "foo"            -> "foo(1)"
/// - "foo.bar"        -> "foo(1).bar"
/// - "foo.bar.baz"    -> "foo(1).bar.baz"
/// - "foo(1)"         -> "foo(2)"
/// - "foo(1).bar"     -> "foo(2).bar"
/// - "foo(1).bar.baz" -> "foo(2).bar.baz"
pub fn deduplicate_filename(filename: &str) -> String {
    match filename.split_once('.') {
        Some((filestem, extension)) => deduplicate_filestem(filestem) + "." + extension,
        None => deduplicate_filestem(filename)
    }
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

/// Given e.g. "\"foo\"", it will first turn the input into "&quot;foo&quot;",
/// then into "&amp;quot;foo&amp;quot;". When this is rendered in the browser,
/// the second escaping pass is removed again, i.e. people will see this on
/// the site: "&quot;foo&quot;". Used to render embeddable code snippets.
pub fn html_double_escape_inside_attribute(string: &str) -> String {
    html_escape_inside_attribute(string)
        .replace('&', "&amp;")
}

/// Escape e.g. "me&you" so it can be rendered into an attribute,
/// e.g. as <img alt="me&quot;you" src="...">
pub fn html_escape_inside_attribute(string: &str) -> String {
    string.replace('&', "&amp;")
          .replace('<', "&lt;")
          .replace('>', "&gt;")
          .replace('"', "&quot;")
          .replace('\'', "&#39;")
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
        0..=999 => human_size_bytes(bytes),
        1000..=999999 => human_size_kilobytes(bytes),
        1000000..=999999999 => human_size_megabytes(bytes),
        _ => human_size_gigabytes(bytes)
    }
}

fn human_size_bytes(bytes: u64) -> String {
    format!("{bytes} B")
}

fn human_size_gigabytes(bytes: u64) -> String {
    let gigabytes = bytes as f64 / 1000000000.0;

    // TODO: These branches/conditions seem non-sensical,
    //       figure out what the original intent was and adjust.
    if gigabytes.fract() < 0.1 {
        format!("{} GB", gigabytes as u64)
    } else if gigabytes.fract() < 0.01 {
        format!("{gigabytes:.1} GB")
    } else {
        format!("{gigabytes:.2} GB")
    }
}

fn human_size_kilobytes(bytes: u64) -> String {
    let kilobytes = bytes / 1000;
    format!("{kilobytes} kB")
}

fn human_size_megabytes(bytes: u64) -> String {
    let megabytes = bytes as f64 / 1000000.0;
    if megabytes >= 10.0 || megabytes.fract() < 0.1 {
        format!("{} MB", megabytes as u64)
    } else {
        format!("{megabytes:.1} MB")
    }
}

/// E.g. 1 Jan 2000
pub fn localized_date_d_mmm_yyyy(date: NaiveDate, chrono_locale: ChronoLocale) -> String {
    use chrono::format::{Fixed, Item, Numeric, Pad};

    let format = [
        Item::Numeric(Numeric::Day, Pad::None),
        Item::Space(" "),
        Item::Fixed(Fixed::ShortMonthName),
        Item::Space(" "),
        Item::Numeric(Numeric::Year, Pad::None)
    ];

    date
        .format_localized_with_items(format.iter(), chrono_locale)
        .to_string()
}

/// E.g. 1 January 2000
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

/// Reusable, efficient implementation of the annoying &OsStr to String conversion
pub fn string_from_os_str(os_str: &OsStr) -> String {
    os_str.to_string_lossy().into()
}

/// Reusable, efficient implementation of the annoying OsString to String conversion
pub fn string_from_os_string(os_string: OsString) -> String {
    match os_string.into_string() {
        Ok(string) => string,
        Err(os_string) => os_string.to_string_lossy().into()
    }
}

/// Reusable, efficient implementation of the annoying PathBuf to String conversion
pub fn string_from_pathbuf(path: PathBuf) -> String {
    string_from_os_string(path.into_os_string())
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

pub fn url_safe_hash_base64(hashable: &impl Hash) -> String {
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}

/// Checks whether the string contains only a-Z, A-Z, 0-9, dash and underscore
pub fn valid_slug_chars(string: &str) -> bool {
    string.chars().all(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// If the slug comes from user input make sure to trim it before passing it.
/// This checks whether the slug is non-empty and whether it contains only
/// allowed characters.
pub fn validate_slug(string: &str) -> Result<(), String> {
    if string.is_empty() {
        return Err(String::from("Can't be empty"));
    }

    if !valid_slug_chars(string) {
        let slugified = slugify(string);
        return Err(format!("Not a slug, allowed is e.g. {slugified}"));
    }

    Ok(())
}

// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::{OsStr, OsString};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use nanoid::nanoid;

use hyper8_translations::Translations;

pub mod ffs;

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

pub fn url_safe_hash_base64(content: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}

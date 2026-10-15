// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WebVTT timecode formats:
//! - mm:ss.ttt
//! - hh:mm:ss.ttt
//!
//! See these resources:
//! - https://www.w3.org/TR/webvtt1/#webvtt-timestamp
//! - https://developer.mozilla.org/en-US/docs/Web/API/WebVTT_API/Web_Video_Text_Tracks_Format#cue_timings
//!
//! SubRip timecode format (same as WebVTT except comma (,) used as fractional part separator):
//! 00:03:18,608 --> 00:03:20,371
//!
//! See: https://en.wikipedia.org/wiki/SubRip
//!
//! For our own native timecode format see the design notes in `ARCHITECTURE.md`.

// TODO: Q: Do we want subchapters?
//
// W3C says on this:
// "A WebVTT chapter cue is a WebVTT cue whose cue text is interpreted as a
//  chapter title that describes the chapter as a navigation target. Chapter
//  cues mark up the timeline of a audio or video file in consecutive,
//  non-overlapping intervals. It is further possible to subdivide these
//  intervals into sub-chapters building a navigation tree."
// (see https://www.w3.org/TR/webvtt1/#chapter-cues)
//
// A: Probably not – very complex implementation, probably very few usecases
// out in the wild. Not even YouTube supports this (probably)?

/// A number component in a timecode string, for instance the "14(m)"
/// in "14m3s" or "14(:)" in "14:03.000". The `remainder_offset` is the
/// absolute byte offset at which the parsed component ends in the string
/// from which the component was parsed.
#[derive(Debug)]
pub struct Component<'a> {
    pub number: u16,
    pub number_len: usize,
    pub remainder_offset: usize,
    pub separator: &'a str
}

#[derive(Clone, Debug)]
pub struct Timecode {
    pub representation: String,
    pub seconds: f32
}

impl<'a> Component<'a> {
    /// Takes a string value like "14m3s Chapter title", a byte offset into
    /// that string at which it should begin processing, and one or more
    /// separator patterns. At the given offset it begins to parse a
    /// number (e.g. 14 or 3) followed by one of the separators. "" is a
    /// special case separator (no separator), in which case the matching
    /// happens against one or more mandatory whitespace characters following
    /// the number. Note that separators such as "ms" and "m" must be passed
    /// in exactly that order as otherwise the "m" will match before
    /// the "ms".
    pub fn parse(
        offset: usize,
        separators: &'a[&str],
        value: &'a str
    ) -> Option<Component<'a>> {
        // Determine number of leading digits
        let number_len = value[offset..].find(|c: char| !c.is_ascii_digit())?;

        // No leading digits
        if number_len == 0 { return None; }

        // Parse leading digits into number
        let number = value[offset..(offset + number_len)].parse::<u16>().ok()?;

        for separator in separators {
            if *separator == "" {
                if !value[(offset + number_len)..].starts_with(|c: char| !c.is_whitespace()) {
                    let whitespace_len = value[(offset + number_len)..]
                        .find(|c: char| !c.is_ascii_whitespace())
                        .unwrap_or(value[(offset + number_len)..].len());

                    let remainder_offset = offset + number_len + whitespace_len;

                    let component = Component {
                        number,
                        number_len,
                        remainder_offset,
                        separator
                    };

                    return Some(component);
                }
            } else if value[(offset + number_len)..].starts_with(separator) {
                let remainder_offset = offset + number_len + separator.len();

                let component = Component {
                    number,
                    number_len,
                    remainder_offset,
                    separator
                };

                return Some(component);
            }
        }

        None
    }
}

impl Timecode {
    fn new(representation: impl Into<String>, seconds: f32) -> Timecode {
        let representation = representation.into();

        Timecode {
            representation,
            seconds
        }
    }

    /// Parses and returns the timecode and remainder of the string (separated
    /// by one or more whitespace characters)
    pub fn parse(value: &str) -> Option<(Timecode, &str)> {
        // "XX:" (SRT/WebVTT)
        // "Xh" | "Xm" | "Xs" | "Xms" | "X " (Hypertime)
        let first_component = Component::parse(0, &[":", "ms", "h", "m", "s", ""], value)?;

        if first_component.number_len == 2 && first_component.separator == ":" {
            // "XX:XX:" (SRT/WebVTT)
            // "XX:XX." (WebVTT)
            let second_component = Component::parse(first_component.remainder_offset, &[":", "."], value)?;

            // Too few/many digits
            if second_component.number_len != 2 { return None; }

            if second_component.separator == ":" {
                // "XX:XX:XX," (SRT)
                // "XX:XX:XX." (WebVTT)
                let third_component = Component::parse(second_component.remainder_offset, &[",", "."], value)?;

                // Too few/many digits
                if third_component.number_len != 2 { return None; }

                // "XX:XX:XX,XXX " (SRT)
                // "XX:XX:XX.XXX " (WebVTT)
                let fourth_component = Component::parse(third_component.remainder_offset, &[""], value)?;

                // Too few/many digits
                if fourth_component.number_len != 3 { return None; }

                // Invalid minute/second components
                if second_component.number > 59 || third_component.number > 59 {
                    return None;
                }

                let seconds =
                    first_component.number as f32 * 3600.0 +
                    second_component.number as f32 * 60.0 +
                    third_component.number as f32 +
                    fourth_component.number as f32 * 0.001;

                let timecode = Timecode::new(&value[..fourth_component.remainder_offset], seconds);

                Some((timecode, &value[fourth_component.remainder_offset..]))
            } else /* if second_component.separator == "." */ {
                // "XX:XX.XXX " (WebVTT)
                let third_component = Component::parse(second_component.remainder_offset, &[""], value)?;

                // Too few/many digits
                if third_component.number_len != 3 { return None; }

                // Invalid minute/second components
                if first_component.number > 59 || second_component.number > 59 {
                    return None;
                }

                let seconds =
                    first_component.number as f32 * 60.0 +
                    second_component.number as f32 +
                    third_component.number as f32 * 0.001;

                let timecode = Timecode::new(&value[..third_component.remainder_offset], seconds);

                Some((timecode, &value[third_component.remainder_offset..]))
            }
        } else {
            // TODO: Unclarity in the specification for "Hypertime" – should such values be allowed?
            // - 90m (quite reasonable)
            // - 90m30s (slightly suspicious)
            // - 1h90m (very odd)

            enum LastHandled {
                Hours,
                Minutes,
                None,
                Seconds
            }

            fn handle_hours_or_next<'a>(
                component: Component,
                value: &'a str
            ) -> Option<(Timecode, &'a str)> {
                if component.separator == "h" {
                    let seconds = component.number as f32 * 3600.0;

                    if value[component.remainder_offset..].starts_with(|c: char| !c.is_whitespace()) {
                        // "…Xm" | "…Xs" | "…Xms" | "…X" (Hypertime)
                        let next_component = Component::parse(component.remainder_offset, &["ms", "m", "s", ""], value)?;

                        handle_minutes_or_next(LastHandled::Hours, seconds, next_component, value)
                    } else {
                        let remainder = value[component.remainder_offset..].trim_start();
                        let timecode = Timecode::new(&value[..component.remainder_offset], seconds);

                        Some((timecode, remainder))
                    }
                } else {
                    handle_minutes_or_next(LastHandled::None, 0.0, component, value)
                }
            }

            fn handle_minutes_or_next<'a>(
                last_handled: LastHandled,
                previous_seconds: f32,
                component: Component,
                value: &'a str
            ) -> Option<(Timecode, &'a str)> {
                if component.separator == "m" {
                    let seconds = previous_seconds + component.number as f32 * 60.0;

                    if value[component.remainder_offset..].starts_with(|c: char| !c.is_whitespace()) {
                        // "…Xs" | "…Xms" | "…X" (Hypertime)
                        let next_component = Component::parse(component.remainder_offset, &["ms", "s", ""], value)?;

                        handle_seconds_or_next(LastHandled::Minutes, seconds, next_component, value)
                    } else {
                        let remainder = value[component.remainder_offset..].trim_start();
                        let timecode = Timecode::new(&value[..component.remainder_offset], seconds);

                        Some((timecode, remainder))
                    }
                } else {
                    handle_seconds_or_next(last_handled, previous_seconds, component, value)
                }
            }

            fn handle_seconds_or_next<'a>(
                last_handled: LastHandled,
                previous_seconds: f32,
                component: Component,
                value: &'a str
            ) -> Option<(Timecode, &'a str)> {
                if component.separator == "s" {
                    let seconds = previous_seconds + component.number as f32;

                    if value[component.remainder_offset..].starts_with(|c: char| !c.is_whitespace()) {
                        // "…Xms" | "…X" (Hypertime)
                        let next_component = Component::parse(component.remainder_offset, &["ms", ""], value)?;

                        handle_millis_or_implicit_seconds(LastHandled::Seconds, seconds, next_component, value)
                    } else {
                        let remainder = value[component.remainder_offset..].trim_start();
                        let timecode = Timecode::new(&value[..component.remainder_offset], seconds);

                        Some((timecode, remainder))
                    }
                } else {
                    handle_millis_or_implicit_seconds(last_handled, previous_seconds, component, value)
                }
            }

            fn handle_millis_or_implicit_seconds<'a>(
                last_handled: LastHandled,
                previous_seconds: f32,
                component: Component,
                value: &'a str
            ) -> Option<(Timecode, &'a str)> {
                if component.separator == "ms" {
                    let seconds = previous_seconds + component.number as f32 * 0.001;

                    if value[component.remainder_offset..].starts_with(|c: char| !c.is_whitespace()) {
                        // Nothing may follow after "…Xms"
                        None
                    } else {
                        let remainder = value[component.remainder_offset..].trim_start();
                        let timecode = Timecode::new(&value[..component.remainder_offset], seconds);

                        Some((timecode, remainder))
                    }
                } else /* if component.separator == " " */ {
                    let seconds = match last_handled {
                        // A unit-less number given after hours is interpreted as minutes
                        LastHandled::Hours => previous_seconds + component.number as f32 * 60.0,
                        // A unit-less number given after minutes is interpreted as seconds
                        LastHandled::Minutes => previous_seconds + component.number as f32,
                        // A unit-less number given on its own is interpreted as seconds
                        LastHandled::None => previous_seconds + component.number as f32,
                        // A unit-less number given after seconds is interpreted as milliseconds
                        LastHandled::Seconds => previous_seconds + component.number as f32 * 0.001,
                    };

                    let timecode = Timecode::new(&value[..component.remainder_offset], seconds);
                    Some((timecode, &value[component.remainder_offset..]))
                }
            }

            handle_hours_or_next(first_component, value)
        }
    }
}

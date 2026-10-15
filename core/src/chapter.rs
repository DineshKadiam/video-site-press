// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::Timecode;

#[derive(Clone, Debug)]
pub struct Chapter {
    pub begin: Timecode,
    pub end: Option<Timecode>,
    pub title: String
}

impl TryFrom<&str> for Chapter {
    type Error = String;

    fn try_from(value: &str) -> Result<Chapter, String> {
        let (begin, remainder) = match Timecode::parse(value) {
            Some((timecode, remainder)) => (timecode, remainder),
            None => {
                let message = formatdoc!(r#"
                    The begin timecode in the chapter declaration '{value}' was not recognized.
                    You can use SRT ("hh:mm:ss,fff") WebVTT ("hh:mm:ss.fff", "mm:ss.fff") and Hyper 8 Hypertime ("1m", "1m2s", "1h2m3s", "1h2m3s4ms", etc.) syntax.
                "#);

                return Err(message);
            }
        };

        let (end, remainder) = if let Some(remainder) = remainder.strip_prefix("-->") {
            if let Some(whitespace_len) = remainder.find(|c: char| !c.is_ascii_whitespace()) &&
                whitespace_len > 0
            {
                match Timecode::parse(&remainder[whitespace_len..]) {
                    Some((timecode, remainder)) => (Some(timecode), remainder),
                    None => {
                        let message = formatdoc!(r#"
                            The end timecode in the chapter declaration '{value}' was not recognized.
                            You can use SRT ("hh:mm:ss,fff") WebVTT ("hh:mm:ss.fff", "mm:ss.fff") and Hyper 8 Hypertime ("1m", "1m2s", "1h2m3s", "1h2m3s4ms", etc.) syntax.
                        "#);

                        return Err(message);
                    }
                }
            } else {
                let message = formatdoc!(r#"
                    The timecode separator in the chapter declaration '{value}' must be surrounded by at least one space/whitespace (but isn't).
                "#);

                return Err(message);
            }
        } else {
            (None, remainder)
        };

        let title = remainder.to_string();

        let chapter = Chapter {
            begin,
            end,
            title
        };

        Ok(chapter)
    }
}

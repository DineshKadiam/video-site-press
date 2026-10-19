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
        if let Some((first_token, remainder)) = value.split_once(|c: char| c.is_whitespace() || c == '-') {
            let begin = if let Ok(timecode) = Timecode::try_from(first_token) {
                 timecode
            } else {
                let message = formatdoc!(r#"
                    The begin timecode '{first_token}' for this chapter was not recognized.
                    You can use SRT ("hh:mm:ss,fff") WebVTT ("hh:mm:ss.fff", "mm:ss.fff") and Hyper 8 Hypertime ("1m", "1m2s", "1h2m3s", "1h2m3s4ms", etc.) syntax.
                "#);

                return Err(message);
            };

            let remainder = remainder.trim_start();

            let (end, remainder) = if let Some(remainder) = remainder.strip_prefix("-->") {
                let remainder = remainder.trim_start();

                if let Some((third_token, remainder)) = remainder.split_once(char::is_whitespace) {
                    let end = if let Ok(timecode) = Timecode::try_from(third_token) {
                        timecode
                    } else {
                        let message = formatdoc!(r#"
                            The end timecode in the chapter declaration '{value}' was not recognized.
                            You can use SRT ("hh:mm:ss,fff") WebVTT ("hh:mm:ss.fff", "mm:ss.fff") and Hyper 8 Hypertime ("1m", "1m2s", "1h2m3s", "1h2m3s4ms", etc.) syntax.
                        "#);

                        return Err(message);
                    };

                    let remainder = remainder.trim_start();

                    (Some(end), remainder)
                } else {
                    let message = format!(r#"Timecode(s) and title in the chapter declaration '{value}' must be separated by at least one space/whitespace (but are not)."#);
                    return Err(message);
                }
            } else {
                (None, remainder)
            };

            if remainder.is_empty() {
                let message = format!(r#"No title was encountered in the chapter declaration '{value}' (but this is mandatory)"#);
                return Err(message);
            }

            let title = remainder.to_string();

            let chapter = Chapter {
                begin,
                end,
                title
            };

            Ok(chapter)
        } else {
            let message = String::from(r#"A chapter must specify at least a begin timecode and title, separated by whitespace, for instance "01:02:35.000 Credits""#);
            Err(message)
        }
    }
}

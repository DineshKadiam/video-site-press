use std::str::Chars;

use crate::util::html_escape_outside_attribute;

pub struct MarkdownSubset<'input> {
    copy_from_index: usize,
    chars: Chars<'input>,
    index: usize,
    input: &'input str,
    output: String
}

impl<'input> MarkdownSubset<'input> {
    fn escape_and_link(string: &str) -> String {
        let mut copy_from_index = 0;
        let mut chars = string.chars();
        let mut index = 0;
        let mut output = String::new();

        let mut word_begin = true;

        loop {
            if word_begin && (chars.as_str().starts_with("https://") || chars.as_str().starts_with("http://")) {
                if index > copy_from_index {
                    output.push_str(&html_escape_outside_attribute(&string[copy_from_index..index]));
                }

                let url_begin_index = index;

                // Skip at least the length of "http://"
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                index += 7;

                let mut url_end_index;

                loop {
                    copy_from_index = index;
                    url_end_index = index;

                    match chars.next() {
                        Some(c) => {
                            index += c.len_utf8();
                            if c == '\n' || c.is_whitespace() {
                                break;
                            }
                        }
                        None => break
                    }
                }

                let href = &string[url_begin_index..url_end_index];
                let link = format!(r#"<a href="{href}" target="_blank">{href}</a>"#);

                output.push_str(&link);
            } else if let Some(c) = chars.next() {
                index += c.len_utf8();
                word_begin = c == '\n' || c.is_whitespace();
            } else {
                break
            }
        }

        if index > copy_from_index {
            output.push_str(&html_escape_outside_attribute(&string[copy_from_index..index]));
        }

        output
    }

    fn new(input: &str) -> MarkdownSubset {
        MarkdownSubset {
            copy_from_index: 0,
            chars: input.chars(),
            index: 0,
            input,
            output: String::new()
        }
    }

    fn next_char(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.index += c.len_utf8();
                Some(c)
            }
            None => None
        }
    }

    /// Converts a limited subset of markdown in the input to html. Additionally,
    /// all user-provided html that is found in the input itself is escaped.
    ///
    /// Right now these conversions are applied:
    /// - `http(s)://example.com` is turned into `<a href="http(s)://example.com" target="_blank">example.com</a>`
    /// - `[Link label][http(s)://example.com]` is turned into `<a href="http(s)://example.com" target="_blank">example.com</a>`
    pub fn to_html(input: &str) -> String {
        let mut parser = MarkdownSubset::new(input);

        loop {
            if parser.chars.as_str().starts_with('[') {
                parser.parse_link();
            } else if parser.chars.as_str().starts_with("\\[") {
                parser.next_char();
                parser.next_char();
            } else if parser.chars.as_str().is_empty() {
                break;
            } else {
                parser.next_char();
            }
        }

        if parser.index > parser.copy_from_index {
            parser.output.push_str(&MarkdownSubset::escape_and_link(&parser.input[parser.copy_from_index..parser.index]));
        }

        parser.output
    }

    fn parse_link(&mut self) {
        let link_begin_index = self.index;

        self.next_char();

        let label_begin_index = self.index;

        loop {
            if self.chars.as_str().starts_with("](") {
                let label_end_index = self.index;

                self.next_char();
                self.next_char();

                if label_end_index == label_begin_index {
                    return;
                }

                let href_begin_index = self.index;

                loop {
                    if self.chars.as_str().starts_with(")") {
                        let href_end_index = self.index;

                        let previous = &self.input[self.copy_from_index..link_begin_index];

                        self.output.push_str(&MarkdownSubset::escape_and_link(previous));

                        let href = &self.input[href_begin_index..href_end_index].replace("\\)", ")");
                        let label = html_escape_outside_attribute(&self.input[label_begin_index..label_end_index].replace("\\]", "]"));
                        let link = format!(r#"<a href="{href}" target="_blank">{label}</a>"#);

                        self.output.push_str(&link);

                        self.next_char();
                        self.copy_from_index = self.index;
                    } else if self.chars.as_str().starts_with("\\)") {
                        self.chars.next();
                        self.chars.next();
                        self.index += 2;
                    } else if self.chars.as_str().starts_with("\n") || self.chars.as_str().is_empty() {
                        return
                    } else {
                        self.chars.next();
                        self.index += 1;
                    }
                }
            } else if self.chars.as_str().starts_with("\\]") {
                self.chars.next();
                self.chars.next();
                self.index += 2;
            } else if self.chars.as_str().starts_with(']') || self.chars.as_str().starts_with('\n') {
                self.next_char();
                return;
            } else {
                self.next_char();
            }
        }
    }
}

#[test]
fn check_explicit_link() {
    let input = "[Label](https://example.com)";
    let output = r#"<a href="https://example.com" target="_blank">Label</a>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_explicit_link_inbetween() {
    let input = "before[Label](https://example.com)after";
    let output = r#"before<a href="https://example.com" target="_blank">Label</a>after"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_implicit_link() {
    let input = "https://example.com";
    let output = r#"<a href="https://example.com" target="_blank">https://example.com</a>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_implicit_link_inbetween() {
    let input = "before https://example.com after";
    let output = r#"before <a href="https://example.com" target="_blank">https://example.com</a> after"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_mixed() {
    let input = "<Hello> [<World>](https://example.com)\nCheck this text & this address: https://example.com";
    let output = "&lt;Hello&gt; <a href=\"https://example.com\" target=\"_blank\">&lt;World&gt;</a>\nCheck this text &amp; this address: <a href=\"https://example.com\" target=\"_blank\">https://example.com</a>";

    assert_eq!(MarkdownSubset::to_html(input), output);
}
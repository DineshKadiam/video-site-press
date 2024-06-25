// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Range;
use std::str::Chars;

use crate::util::html_escape_outside_attribute;

pub struct MarkdownSubset<'input> {
    chars: Chars<'input>,
    index: usize,
    input: &'input str,
    output: String,
    paragraph: Option<Paragraph>
}

#[derive(Clone)]
struct Paragraph {
    /// This does not necessarily reflect the range from begin to end of the paragraph,
    /// e.g. in the case of a link appearing within the paragraph, afterwards the
    /// copy_range is just the content following in the paragraph right after the link.
    copy_range: Range<usize>,
    trailing_newlines: usize,
    trailing_whitespace: usize
}

impl<'input> MarkdownSubset<'input> {
    fn consume_newline(&mut self) {
        self.next_char();

        if let Some(paragraph) = &mut self.paragraph {
            if paragraph.trailing_newlines == 0 && paragraph.trailing_whitespace == 0 {
                paragraph.copy_range.end = self.index;
            }

            paragraph.trailing_newlines += 1;
        }
    }

    fn consume_non_whitespace(&mut self) {
        match &mut self.paragraph {
            Some(paragraph) => {
                if paragraph.trailing_newlines > 1 {
                    if !paragraph.copy_range.is_empty() {
                        self.output.push_str(&MarkdownSubset::escape_and_link(&self.input[paragraph.copy_range.clone()]));
                    }
                    self.output.push_str("</p><p>");
                    paragraph.copy_range = self.index..self.index;
                }

                paragraph.trailing_newlines = 0;
                paragraph.trailing_whitespace = 0;
            }
            None => {
                self.output.push_str("<p>");
                self.paragraph = Some(Paragraph {
                    copy_range: self.index..self.index,
                    trailing_newlines: 0,
                    trailing_whitespace: 0
                })
            }
        }

        self.next_char();
        self.paragraph.as_mut().unwrap().copy_range.end = self.index;
    }

    fn consume_whitespace(&mut self) {
        self.next_char();

        if let Some(paragraph) = &mut self.paragraph {
            if paragraph.trailing_newlines == 0 && paragraph.trailing_whitespace == 0 {
                paragraph.copy_range.end = self.index;
            }

            paragraph.trailing_whitespace += 1;
        }
    }

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
            chars: input.chars(),
            index: 0,
            input,
            output: String::new(),
            paragraph: None
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
    /// - Text is split into paragraphs, based on separation by any number of empty lines in between the text
    /// - `http(s)://example.com` is turned into `<a href="http(s)://example.com" target="_blank">example.com</a>`
    /// - `[Link label][http(s)://example.com]` is turned into `<a href="http(s)://example.com" target="_blank">example.com</a>`
    pub fn to_html(input: &str) -> String {
        let mut parser = MarkdownSubset::new(input);

        loop {
            if parser.chars.as_str().starts_with('[') {
                parser.parse_link();
            } else if parser.chars.as_str().starts_with("\\[") {
                parser.consume_non_whitespace();
                parser.consume_non_whitespace();
            } else if parser.chars.as_str().starts_with('\n') {
                parser.consume_newline();
            } else if parser.chars.as_str().starts_with(|c: char| c.is_whitespace()) {
                parser.consume_whitespace();
            } else if parser.chars.as_str().is_empty() {
                break;
            } else {
                parser.consume_non_whitespace();
            }
        }

        if let Some(paragraph) = parser.paragraph {
            if !paragraph.copy_range.is_empty() {
                parser.output.push_str(&MarkdownSubset::escape_and_link(&parser.input[paragraph.copy_range]));
            }
            parser.output.push_str("</p>");
        }

        parser.output
    }

    fn parse_link(&mut self) {
        let paragraph_before_link = self.paragraph.clone();

        self.consume_non_whitespace();

        let label_begin_index = self.index;

        loop {
            if self.chars.as_str().starts_with("](") {
                let label_end_index = self.index;

                self.consume_non_whitespace();
                self.consume_non_whitespace();

                if label_end_index == label_begin_index {
                    return;
                }

                let href_begin_index = self.index;

                loop {
                    if self.chars.as_str().starts_with(")") {
                        let href_end_index = self.index;

                        if let Some(paragraph) = paragraph_before_link {
                            if !paragraph.copy_range.is_empty() {
                                let previous = &self.input[paragraph.copy_range];
                                self.output.push_str(&MarkdownSubset::escape_and_link(previous));
                            }
                        }

                        let href = &self.input[href_begin_index..href_end_index].replace("\\)", ")");
                        let label = html_escape_outside_attribute(&self.input[label_begin_index..label_end_index].replace("\\]", "]"));
                        let link = format!(r#"<a href="{href}" target="_blank">{label}</a>"#);

                        self.output.push_str(&link);

                        self.next_char();
                        self.paragraph.as_mut().unwrap().copy_range = self.index..self.index;

                        return;
                    } else if self.chars.as_str().starts_with("\\)") {
                        self.consume_non_whitespace();
                        self.consume_non_whitespace();
                    } else if self.chars.as_str().starts_with('\n') {
                        self.consume_newline();
                        return;
                    } else if self.chars.as_str().is_empty() {
                        return;
                    } else {
                        self.consume_non_whitespace();
                    }
                }
            } else if self.chars.as_str().starts_with("\\]") {
                self.consume_non_whitespace();
                self.consume_non_whitespace();
            } else if self.chars.as_str().starts_with(']') {
                self.consume_non_whitespace();
                return;
            } else if self.chars.as_str().starts_with('\n') {
                self.consume_newline();
                return;
            } else if self.chars.as_str().is_empty() {
                return;
            } else {
                self.consume_non_whitespace();
            }
        }
    }
}

#[test]
fn check_explicit_link() {
    let input = "[Label](https://example.com)";
    let output = r#"<p><a href="https://example.com" target="_blank">Label</a></p>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_explicit_link_inbetween() {
    let input = "before[Label](https://example.com)after";
    let output = r#"<p>before<a href="https://example.com" target="_blank">Label</a>after</p>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_implicit_link() {
    let input = "https://example.com";
    let output = r#"<p><a href="https://example.com" target="_blank">https://example.com</a></p>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_implicit_link_inbetween() {
    let input = "before https://example.com after";
    let output = r#"<p>before <a href="https://example.com" target="_blank">https://example.com</a> after</p>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_mixed() {
    let input = "<Hello> [<World>](https://example.com)\nCheck this text & this address: https://example.com";
    let output = "<p>&lt;Hello&gt; <a href=\"https://example.com\" target=\"_blank\">&lt;World&gt;</a>\nCheck this text &amp; this address: <a href=\"https://example.com\" target=\"_blank\">https://example.com</a></p>";

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_two_explicit_links() {
    let input = "[Foo](foo) and [Bar](bar)";
    let output = r#"<p><a href="foo" target="_blank">Foo</a> and <a href="bar" target="_blank">Bar</a></p>"#;

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_two_paragraphs() {
    let input = "Enter Alice.\n\nEnter Bob.";
    let output = "<p>Enter Alice.\n</p><p>Enter Bob.</p>";

    assert_eq!(MarkdownSubset::to_html(input), output);
}

#[test]
fn check_two_paragraphs_with_links() {
    let input = "Enter [Alice](https://example.com/alice).\n\nEnter [Bob](https://example.com/bob).";
    let output = "<p>Enter <a href=\"https://example.com/alice\" target=\"_blank\">Alice</a>.\n</p><p>Enter <a href=\"https://example.com/bob\" target=\"_blank\">Bob</a>.</p>";

    assert_eq!(MarkdownSubset::to_html(input), output);
}


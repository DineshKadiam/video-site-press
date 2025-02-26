// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

use crate::Language;
use crate::util::html_escape_outside_attribute;

pub struct Textarea<'a> {
    hint: Option<&'a str>,
    id: Option<&'a str>,
    key: &'a str,
    label: &'a str,
    rows: usize,
    value: String
}

impl<'a> Textarea<'a> {
    pub fn id(&mut self, id: &'a str) -> &mut Textarea<'a> {
        self.id = Some(id);
        self
    }

    /// Initialize a textarea for an image description
    pub fn image_description(key: &'a str, label: &'a str) -> Textarea<'a> {
        Textarea::new(None, key, label, 5)
    }

    /// Initialize a textarea for markdown subset text
    pub fn markdown_subset(language: &Language, key: &'a str, label: &'a str) -> Textarea<'a> {
        let hint = Some(language.translations.markdown_inline_links_are_supported_standalone_urls_are_automatically_linked);
        Textarea::new(hint, key, label, 8)
    }

    fn new(hint: Option<&'a str>, key: &'a str, label: &'a str, rows: usize) -> Textarea<'a> {
        Textarea {
            hint,
            id: None,
            key,
            label,
            rows,
            value: String::new()
        }
    }

    pub fn value(&mut self, value: &str) {
        self.value = html_escape_outside_attribute(value);
    }

    pub fn value_option<V: AsRef<str>>(&mut self, value_option: &Option<V>) {
        if let Some(value) = value_option {
            self.value = html_escape_outside_attribute(value.as_ref());
        }
    }
}

impl Display for Textarea<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let key = self.key;
        let id = if let Some(id) = self.id { id } else { self.key };
        let label = self.label;
        let rows = self.rows;
        let value = &self.value;

        let textarea = formatdoc!(r#"
            <textarea id="{id}" name="{key}" rows="{rows}">{value}</textarea>
        "#);

        let textarea_wrapped = if let Some(hint) = self.hint {
            formatdoc!(r#"
                <div class="textarea_with_hint">
                    <span>{hint}</span>
                    {textarea}
                </div>
            "#)
        } else {
            textarea
        };

        let html = formatdoc!(r#"
            <div class="form_field">
                <label for="{id}">{label}</label>
                {textarea_wrapped}
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

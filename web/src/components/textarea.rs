// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

use hyper8_core::MAX_SYNOPSIS_CHARS;
use hyper8_translations::Translations;
use hyper8_util::html_escape_outside_attribute;

use crate::Field;

pub struct Textarea<'a> {
    error: String,
    hint: Option<String>,
    id: Option<&'a str>,
    key: &'a str,
    label: &'a str,
    limit: Option<usize>,
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
        Textarea::new(
            None,
            key,
            label,
            None,
            5
        )
    }

    /// Initialize a textarea for a manifest
    pub fn manifest(key: &'a str, label: &'a str) -> Textarea<'a> {
        Textarea::new(
            None,
            key,
            label,
            None,
            16
        )
    }

    /// Initialize a textarea for markdown subset text
    pub fn markdown_subset(
        translations: &Translations,
        key: &'a str,
        label: &'a str
    ) -> Textarea<'a> {
        let hint = Some(translations.textarea_formatting_hint_text.to_string());
        Textarea::new(
            hint,
            key,
            label,
            None,
            8
        )
    }

    fn new(
        hint: Option<String>,
        key: &'a str,
        label: &'a str,
        limit: Option<usize>,
        rows: usize
    ) -> Textarea<'a> {
        Textarea {
            error: String::new(),
            hint,
            id: None,
            key,
            label,
            limit,
            rows,
            value: String::new()
        }
    }

    /// Initialize a textarea for a synopsis text
    pub fn synopsis(
        translations: &Translations,
        key: &'a str,
        label: &'a str
    ) -> Textarea<'a> {
        let hint = Some(translations.xxx_characters_maximum(MAX_SYNOPSIS_CHARS));
        Textarea::new(
            hint,
            key,
            label,
            Some(MAX_SYNOPSIS_CHARS),
            3
        )
    }

    pub fn validated_value(&mut self, field: &Field) {
        if let Some(error) = &field.error {
            self.error = format!(r#"<span class="error">{error}</span>"#);
        }

        self.value(&field.value);
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
        let error = &self.error;
        let key = self.key;
        let label = self.label;
        let rows = self.rows;
        let value = &self.value;

        let id = if let Some(id) = self.id { id } else { self.key };

        let maxlength = if let Some(limit) = self.limit {
            format!(r#"maxlength="{limit}""#)
        } else {
            String::new()
        };

        let textarea = formatdoc!(r#"
            <textarea id="{id}" {maxlength} name="{key}" rows="{rows}">{value}</textarea>
        "#);

        let textarea_wrapped = if let Some(hint) = &self.hint {
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
                {error}
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

use hyper8_core::{Site, SitePath, SiteUrl};
use hyper8_translations::Translations;
use hyper8_util::html_escape_inside_attribute;

use crate::Field;

pub struct Input<'a> {
    error: String,
    id: Option<&'a str>,
    key: &'a str,
    label: &'a str,
    required: bool,
    r#type: InputType,
    value: String
}

#[derive(PartialEq)]
enum InputType {
    Date,
    Number,
    Password,
    Permalink {
        prefix: String
    },
    Text,
    Url
}

impl<'a> Input<'a> {
    /// Initialize an input for a date
    pub fn date(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Date)
    }

    pub fn id(&mut self, id: &'a str) -> &mut Input<'a> {
        self.id = Some(id);
        self
    }

    fn new(key: &'a str, label: &'a str, r#type: InputType) -> Input<'a> {
        Input {
            error: String::new(),
            id: None,
            key,
            label,
            required: false,
            r#type,
            value: String::new()
        }
    }

    /// Initialize an input for a number
    pub fn number(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Number)
    }

    /// Initialize an input for password
    pub fn password(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Password)
    }

    /// Initialize an input for a permalink
    pub fn permalink(
        key: &'a str,
        parent_site_path: &SitePath,
        site: &Site,
        translations: &'a Translations
    ) -> Input<'a> {
        let prefix = site.absolute_url_prefix(parent_site_path)
            .unwrap_or_else(|| SiteUrl::example_url().join_prefix(parent_site_path));

        Input::new(key, translations.permalink, InputType::Permalink { prefix })
    }

    pub fn required(&mut self) -> &mut Input<'a> {
        self.required = true;
        self
    }

    /// Initialize an input for text
    pub fn text(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Text)
    }

    /// Initialize an input for a url
    pub fn url(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Url)
    }

    pub fn validated_value(&mut self, field: &Field) {
        if let Some(error) = &field.error {
            self.error = format!(r#"<span class="error">{error}</span>"#);
        }

        self.value(&field.value);
    }

    pub fn value(&mut self, value: &str) {
        let value_escaped = html_escape_inside_attribute(value);
        self.value = format!(r#"value="{value_escaped}""#);
    }

    pub fn value_option<V: AsRef<str>>(&mut self, value_option: &Option<V>) {
        if let Some(value) = value_option {
            let value_escaped = html_escape_inside_attribute(value.as_ref());
            self.value = format!(r#"value="{value_escaped}""#);
        }
    }
}

impl Display for Input<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let error = &self.error;
        let key = self.key;
        let id = if let Some(id) = self.id { id } else { self.key };
        let label = self.label;
        let required = if self.required { "required" } else { "" };
        let r#type = match self.r#type {
            InputType::Date => r#"type="date""#,
            InputType::Number => r#"type="number""#,
            InputType::Password => r#"type="password""#,
            InputType::Permalink { .. } => "",
            InputType::Text => "",
            InputType::Url => r#"type="url""#
        };
        let value = &self.value;

        let input = if let InputType::Permalink { prefix } = &self.r#type {
            // The &lrm; at the end of the prefix solves the trailing slash being
            // cut off by text-overflow: ellipsis in conjunction with direction: rtl
            formatdoc!(r#"
                <div class="permalink_split">
                    <span class="input_prefix">{prefix}&lrm;</span>
                    <input autocomplete="off" id="{id}" name="{key}" pattern="[A-Za-z0-9\-]+" {required} {type} {value}>
                </div>
            "#)
        } else {
            format!(r#"<input id="{id}" name="{key}" {required} {type} {value}>"#)
        };

        let html = formatdoc!(r#"
            <div class="form_field">
                <label for="{id}">{label}</label>
                {input}
                {error}
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

use crate::Field;
use crate::util::html_escape_inside_attribute;

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
    Permalink,
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

    /// Initialize an input for a permalink
    pub fn permalink(key: &'a str, label: &'a str) -> Input<'a> {
        Input::new(key, label, InputType::Permalink)
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

impl<'a> Display for Input<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let autocomplete = if self.r#type == InputType::Permalink { r#"autocomplete="off""# } else { "" };
        let error = &self.error;
        let key = self.key;
        let id = if let Some(id) = self.id { id } else { self.key };
        let label = self.label;
        let pattern = if self.r#type == InputType::Permalink { r#"pattern="[A-Za-z0-9\-]+""# } else { "" };
        let required = if self.required { "required" } else { "" };
        let r#type = match self.r#type {
            InputType::Date => r#"type="date""#,
            InputType::Permalink => "",
            InputType::Text => "",
            InputType::Url => r#"type="url""#
        };
        let value = &self.value;

        let html = formatdoc!(r#"
            <div class="form_field">
                <label for="{id}">{label}</label>
                <input {autocomplete} id="{id}" name="{key}" {pattern} {required} {type} {value}>
                {error}
            </div>
        "#);

        write!(formatter, "{html}")
    }
}
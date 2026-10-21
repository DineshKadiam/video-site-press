// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use crate::html_escape_inside_attribute;

/// A helper abstraction for collecting and rendering attributes for html
/// tags. Initialize with `new`, then add attributes with the `add_escaped`
/// and `add_raw` methods. The attributes can directly be interpolated into
/// strings because this abstraction implements [Display].
pub struct Attributes {
    string: String
}

impl Attributes {
    /// Internally used to add key="value" to the attributes
    fn add(&mut self, key: &str, value: &str) {
        if !self.string.is_empty() {
            self.string.push(' ');
        }

        self.string.push_str(key);
        self.string.push_str("=\"");
        self.string.push_str(value);
        self.string.push('"');
    }

    /// Add a key="value" attribute and escape the value
    pub fn add_escaped(&mut self, key: &str, value: impl AsRef<str>) {
        let value_escaped = html_escape_inside_attribute(value.as_ref());
        self.add(key, &value_escaped);
    }

    /// Add a key-only attribute such as "disabled", "data-something", etc.
    pub fn add_key(&mut self, key: &str) {
        if !self.string.is_empty() {
            self.string.push(' ');
        }

        self.string.push_str(key);
    }

    /// Add a key="value" attribute without escaping the value
    pub fn add_raw(&mut self, key: &str, value: impl AsRef<str>) {
        self.add(key, value.as_ref());
    }

    /// Initialize an empty attributes string
    pub fn new() -> Attributes {
        Attributes {
            string: String::new()
        }
    }
}

impl Display for Attributes {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.string)
    }
}

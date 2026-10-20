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

    pub fn add_escaped(&mut self, key: &str, value: impl AsRef<str>) {
        let value_escaped = html_escape_inside_attribute(value.as_ref());
        self.add(key, &value_escaped);
    }

    pub fn add_raw(&mut self, key: &str, value: impl AsRef<str>) {
        self.add(key, value.as_ref());
    }

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

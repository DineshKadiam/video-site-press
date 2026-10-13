// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

pub struct Checkbox<'a> {
    checked: bool,
    key: &'a str,
    label: &'a str,
}

impl<'a> Checkbox<'a> {
    pub fn checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn new(key: &'a str, label: &'a str) -> Checkbox<'a> {
        Checkbox {
            checked: false,
            key,
            label
        }
    }
}

impl Display for Checkbox<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let checked = if self.checked { "checked" } else { "" };
        let key = self.key;
        let label = self.label;

        let html = formatdoc!(r#"
            <div class="form_field">
                <div>
                    <input {checked} id="{key}" name="{key}" type="checkbox">
                    <label for="{key}">{label}</label>
                </div>
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

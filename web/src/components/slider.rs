// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

pub struct Slider<'a, T: Copy + Display> {
    default: T,
    key: &'a str,
    label: &'a str,
    min: T,
    max: T,
    value: Option<T>
}

impl<'a, T: Copy + Display> Slider<'a, T> {
    pub fn new(
        default: T,
        key: &'a str,
        label: &'a str,
        max: T,
        min: T
    ) -> Slider<'a, T> {
        Slider {
            default,
            key,
            label,
            max,
            min,
            value: None
        }
    }

    pub fn value(&mut self, value: Option<T>) {
        self.value = value;
    }
}

impl<T: Copy + Display> Display for Slider<'_, T> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let checked = if self.value.is_some() { "checked" } else { "" };
        let disabled = if self.value.is_none() { "disabled" } else { "" };
        let key = self.key;
        let label = self.label;
        let max = self.max;
        let min = self.min;
        let value = &self.value.unwrap_or(self.default);

        // TODO: Figure out how to label/make this accessible best (having
        // three inputs for one setting)
        let html = formatdoc!(r#"
            <div class="form_field slider">
                <div>
                    <label for="{key}_enabled">{label}</label>
                    <input {disabled} key="{key}_value" max="{max}" min="{min}" name="{key}_value" type="number" value="{value}">
                </div>
                <div>
                    <input {checked} key="{key}_enabled" name="{key}_enabled" type="checkbox">
                    <input {disabled} key="{key}_slider" max="{max}" min="{min}" type="range" value="{value}">
                </div>
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use indoc::formatdoc;

pub struct Select<'a> {
    id: Option<&'a str>,
    key: &'a str,
    label: &'a str,
    options: Vec<SelectOption>,
    selected: Option<usize>
}

pub struct SelectOption {
	pub key: String,
	pub label: String
}

impl<'a> Select<'a> {
    pub fn id(&mut self, id: &'a str) -> &mut Select<'a> {
        self.id = Some(id);
        self
    }

    pub fn new(key: &'a str, label: &'a str) -> Select<'a> {
        Select {
            id: None,
            key,
            label,
            options: Vec::new(),
            selected: None
        }
    }

    pub fn option<K: Into<String>, V: Into<String>>(&mut self, key: K, label: V) {
    	let option = SelectOption::new(key.into(), label.into());
    	self.options.push(option);
    }

	pub fn selected(&mut self, key: &str) {
		for (index, option) in self.options.iter().enumerate() {
			if option.key == key {
				self.selected = Some(index);
			}
		}
    }
}

impl<'a> Display for Select<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let key = self.key;
        let id = if let Some(id) = self.id { id } else { self.key };
        let label = self.label;

        let options = self.options
        	.iter()
        	.enumerate()
        	.map(|(index, option)| {
        		let key = &option.key;
        		let label = &option.label;
        		let selected = match self.selected.is_some_and(|selected_index| index == selected_index) {
        			true => "selected",
        			false => ""
        		};
        		format!(r#"<option {selected} value="{key}">{label}</option>"#)
        	})
        	.collect::<Vec<String>>()
        	.join("\n");

        let html = formatdoc!(r#"
            <div class="form_field">
                <label for="{id}">{label}</label>
                <select id="{id}" name="{key}">
                	{options}
                </select>
            </div>
        "#);

        write!(formatter, "{html}")
    }
}

impl SelectOption {
    pub fn new(key: String, label: String) -> SelectOption {
        SelectOption {
            key,
            label
        }
    }
}

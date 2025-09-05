// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub struct Link {
    /// Used in conjunction with rel="me" linking, when we want the link to be present
    /// to verify identity, but not display it.
    pub hidden: bool,
    pub label: Option<String>,
    /// Indicates rel="me" linking (https://microformats.org/wiki/rel-me)
    pub rel_me: bool,
    pub url: String
}

impl Link {
    pub fn new(
        hidden: bool,
        label: Option<String>,
        rel_me: bool,
        url: impl Into<String>
    ) -> Link {
        Link {
            hidden,
            label,
            rel_me,
            url: url.into()
        }
    }

    /// Returns either the label itself, or as a fallback the url
    /// without the http(s):// part and without trailing slash.
    pub fn pretty_label(&self) -> String {
        match &self.label {
            Some(label) => label.clone(),
            None => self.url
                .split_once("://")
                .unwrap()
                .1
                .trim_end_matches('/')
                .to_string()
        }
    }

    /// Serializes the link as an eno field, ready to be written to a collection,
    /// playlist or video manifest.
    pub fn to_eno(&self) -> String {
        let mut field = String::from("link:\n");

        if let Some(label) = &self.label {
            field.push_str(&format!("label = {}\n", label));
        }

        field.push_str(&format!("url = {}\n", self.url));

        if self.rel_me {
            if self.hidden {
                field.push_str("verification = rel-me-hidden\n");
            } else {
                field.push_str("verification = rel-me\n");
            }
        }

        field
    }
}

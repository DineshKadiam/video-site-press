// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use enolib::{Attribute, Element};

pub const MAX_SYNOPSIS_CHARS: usize = 256;

// TODO: Use attribute.snippet(), i.e. make this a attribute_error_with_snippet() (?) – Need to make sure this nicely displays also in interface
pub fn attribute_error(
    attribute: &Attribute,
    manifest_path: &Path,
    error: &str
) -> String {
    format!("Error in {}:{}:\n\n{}", manifest_path.display(), attribute.line_number(), error)
}

// TODO: Use element.snippet(), i.e. make this a element_error_with_snippet() (?) – Need to make sure this nicely displays also in interface
pub fn element_error(
    element: &Element,
    manifest_path: &Path,
    error: &str
) -> String {
    format!("Error in {}:{}:\n\n{}", manifest_path.display(), element.line_number(), error)
}

pub fn not_supported_error(
    manifest_name: &str,
    option_key: &str,
    supported_options_groups: &[&[&str]]
) -> String {
    let mut supported_options_sorted = Vec::new();

    for supported_option_group in supported_options_groups {
        for option in *supported_option_group {
            supported_options_sorted.push(*option);
        }
    }

    supported_options_sorted.sort();

    let r_supported_options = supported_options_sorted.join(", ");

    format!("This '{option_key}' option was not recognized (check for typos and that the option is supported inside a(n) {manifest_name} manifest).\n\nInside this manifest ({manifest_name}) the following keys are supported:\n{r_supported_options}")
}

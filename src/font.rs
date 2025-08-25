// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Debug)]
pub enum Font {
	/// Wraps a file name
	Custom(String),
	Default
}

impl Font {
	pub fn family(&self) -> &str {
		match self {
			Font::Custom(_) => "'Custom'",
			Font::Default => "Inter, Roboto, 'Helvetica Neue', 'Arial Nova', 'Nimbus Sans', Arial, sans-serif"
		}
	}
}
// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use language_tags::LanguageTag;

#[derive(Clone, Debug)]
pub struct SubtitleFile {
	pub explicitly_defined: bool,
	pub file_name: String,
	pub label: String,
	/// Specified as BCP 47 code (e.g. 'en')
	/// (see e.g. https://www.rfc-editor.org/info/bcp47)
	pub language: Option<String>
}

impl SubtitleFile {
	/// Ensure the language is already validated when passing it here.
	pub fn from_manifest(
		file_name: String,
		label_option: Option<String>,
		language: Option<String>
	) -> SubtitleFile {
		let label = label_option.unwrap_or_else(||
			Path::new(&file_name).file_stem().unwrap().to_string_lossy().to_string()
		);

		SubtitleFile {
			explicitly_defined: true,
			file_name,
			label,
			language
		}
	}

	pub fn from_path(path: &Path) -> SubtitleFile {
		let file_name = path.file_name().unwrap().to_string_lossy().to_string();
		let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

		// If the file stem is a valid BCP 47 language code we automatically derive the language from it
		let language = match LanguageTag::parse(&file_stem).is_ok_and(|language_tag| language_tag.is_valid()) {
			true => Some(file_stem.clone()),
			false => None
		};

		SubtitleFile {
			explicitly_defined: false,
			file_name,
			label: file_stem,
			language
		}
	}
}

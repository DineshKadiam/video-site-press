use std::path::Path;

use language_tags::LanguageTag;

#[derive(Clone, Debug)]
pub struct SubtitleFile {
	pub file_name: String,
	/// Specified as BCP 47 code (e.g. 'en')
	/// (see e.g. https://www.rfc-editor.org/info/bcp47)
	pub language: Option<String>
}

impl SubtitleFile {
	pub fn new(path: &Path) -> SubtitleFile {
		let file_name = path.file_name().unwrap().to_string_lossy().to_string();
		let file_stem = path.file_stem().unwrap().to_string_lossy();

		let language = match LanguageTag::parse(&file_stem) {
			Ok(language_tag) => Some(language_tag.to_string()),
			Err(_) => {
				// Entirely ok, using a BCP 47 code as file stem is just an
				// option, but not required at all - the language code can
				// also be specified through the manifest.
				None
			}
		};

		SubtitleFile {
			file_name,
			language
		}
	}
}

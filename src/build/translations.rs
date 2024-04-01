#[derive(Clone, Debug)]
pub struct Translations {
	pub copied: String,
	pub copy_link: String,
	pub failed: String,
	pub videos: String
}

impl Translations {
	pub fn de() -> Translations {
		Translations {
			copied: String::from("Kopiert"),
			copy_link: String::from("Link kopieren"),
			failed: String::from("Fehlgeschlagen"),
			videos: String::from("Videos")
		}
	}

	pub fn en() -> Translations {
		Translations {
			copied: String::from("Copied"),
			copy_link: String::from("Copy link"),
			failed: String::from("Failed"),
			videos: String::from("Videos")
		}
	}
}

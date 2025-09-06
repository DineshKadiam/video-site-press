// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

pub struct Translations {
	pub add_site: &'static str,
	pub no_sites: &'static str
}

impl Translations {
	pub const DE: Translations = Translations {
		add_site: "Seite hinzufügen",
		no_sites: "Keine Seiten"
	};

	pub const EN: Translations = Translations {
		add_site: "Add site",
		no_sites: "No sites"
	};

	pub fn from_code(code: &str) -> Translations {
		match code {
			"de" => Translations::DE,
			"en" => Translations::EN,
			_ => Translations::EN
		}
	}
}

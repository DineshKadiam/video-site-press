#[derive(Clone, Debug)]
pub struct EditorTranslations {
	pub deploy: String,
	pub deploy_configuration_missing_required_fields: String,
	pub filter: String,
	pub preview: String,
	pub really_deploy: String
}

impl EditorTranslations {
	pub fn de() -> EditorTranslations {
		EditorTranslations {
			deploy: String::from("Deployen"),
			deploy_configuration_missing_required_fields: String::from("Erforderliche Felder in der Deploy Konfiguration fehlen"),
			filter: String::from("Filtern"),
			preview: String::from("Vorschau"),
			really_deploy: String::from("Wirklich deployen?")
		}
	}

	pub fn en() -> EditorTranslations {
		EditorTranslations {
			deploy: String::from("Deploy"),
			deploy_configuration_missing_required_fields: String::from("Deploy configuration is missing required fields"),
			filter: String::from("Filter"),
			preview: String::from("Preview"),
			really_deploy: String::from("Really deploy?")
		}
	}

	pub fn from_code(language: &str) -> EditorTranslations {
		match language {
			"de" => EditorTranslations::de(),
			"en" => EditorTranslations::en(),
			_ => {
				eprintln!("Unknown editor language '{language}', falling back to english");
				EditorTranslations::en()
			}
		}
	}
}

#[derive(Clone, Debug)]
pub struct Translations {
	pub copied: String,
	pub copy_link: String,
	pub dark_color_scheme: String,
	pub failed: String,
	pub hyper8_logo: String,
	pub light_color_scheme: String,
	pub loud_volume: String,
	pub maximize: String,
	pub medium_volume: String,
	pub minimize: String,
	pub pause: String,
	pub play: String,
	pub sound_muted: String,
	pub video_not_yet_available: String,
	pub videos: String
}

impl Translations {
	pub fn de() -> Translations {
		Translations {
			copied: String::from("Kopiert"),
			copy_link: String::from("Link kopieren"),
			dark_color_scheme: String::from("Dunkles Farbschema"),
			failed: String::from("Fehlgeschlagen"),
			hyper8_logo: String::from("Hyper 8 Logo"),
			light_color_scheme: String::from("Helles Farbschema"),
			loud_volume: String::from("Hohe Lautstärke"),
			maximize: String::from("Maximieren"),
			medium_volume: String::from("Mittlere Lautstärke"),
			minimize: String::from("Minimieren"),
			pause: String::from("Pausieren"),
			play: String::from("Abspielen"),
			sound_muted: String::from("Ton stummgeschalten"),
			video_not_yet_available: String::from("Video noch nicht verfügbar."),
			videos: String::from("Videos")
		}
	}

	pub fn en() -> Translations {
		Translations {
			copied: String::from("Copied"),
			copy_link: String::from("Copy link"),
			dark_color_scheme: String::from("Dark Color Scheme"),
			failed: String::from("Failed"),
			hyper8_logo: String::from("Hyper 8 Logo"),
			light_color_scheme: String::from("Light Color Scheme"),
			loud_volume: String::from("Loud Volume"),
			maximize: String::from("Maximize"),
			medium_volume: String::from("Medium Volume"),
			minimize: String::from("Minimize"),
			pause: String::from("Pause"),
			play: String::from("Play"),
			sound_muted: String::from("Sound muted"),
			video_not_yet_available: String::from("Video not yet available."),
			videos: String::from("Videos")
		}
	}
}

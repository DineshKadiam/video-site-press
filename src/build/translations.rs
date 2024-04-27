#[derive(Clone, Debug)]
pub struct Translations {
	pub copied: String,
	pub copy_link: String,
	pub dark_color_scheme: String,
	pub download: String,
	pub failed: String,
	pub hyper8_logo: String,
	pub light_color_scheme: String,
	pub loud_volume: String,
	pub maximize: String,
	pub medium_volume: String,
	pub minimize: String,
	pub next_video: String,
	pub no_audio: String,
	pub no_video: String,
	pub pause: String,
	pub play: String,
	pub replay: String,
	pub sound_muted: String,
	pub subtitles: String,
	pub video_not_yet_available: String,
	pub videos: String
}

impl Translations {
	pub fn de() -> Translations {
		Translations {
			copied: String::from("Kopiert"),
			copy_link: String::from("Link kopieren"),
			dark_color_scheme: String::from("Dunkles Farbschema"),
			download: String::from("Download"),
			failed: String::from("Fehlgeschlagen"),
			hyper8_logo: String::from("Hyper 8 Logo"),
			light_color_scheme: String::from("Helles Farbschema"),
			loud_volume: String::from("Hohe Lautstärke"),
			maximize: String::from("Maximieren"),
			medium_volume: String::from("Mittlere Lautstärke"),
			minimize: String::from("Minimieren"),
			next_video: String::from("Nächstes Video"),
			no_audio: String::from("Kein Audio"),
			no_video: String::from("Kein Video"),
			pause: String::from("Pausieren"),
			play: String::from("Abspielen"),
			replay: String::from("Erneut abspielen"),
			sound_muted: String::from("Ton stummgeschalten"),
			subtitles: String::from("Untertitel"),
			video_not_yet_available: String::from("Video noch nicht verfügbar."),
			videos: String::from("Videos")
		}
	}

	pub fn en() -> Translations {
		Translations {
			copied: String::from("Copied"),
			copy_link: String::from("Copy Link"),
			dark_color_scheme: String::from("Dark Color Scheme"),
			download: String::from("Download"),
			failed: String::from("Failed"),
			hyper8_logo: String::from("Hyper 8 Logo"),
			light_color_scheme: String::from("Light Color Scheme"),
			loud_volume: String::from("Loud Volume"),
			maximize: String::from("Maximize"),
			medium_volume: String::from("Medium Volume"),
			minimize: String::from("Minimize"),
			next_video: String::from("Next Video"),
			no_audio: String::from("No audio"),
			no_video: String::from("No video"),
			pause: String::from("Pause"),
			play: String::from("Play"),
			replay: String::from("Replay"),
			sound_muted: String::from("Sound muted"),
			subtitles: String::from("Subtitles"),
			video_not_yet_available: String::from("Video not yet available."),
			videos: String::from("Videos")
		}
	}
}

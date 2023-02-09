#[derive(Clone, Debug)]
pub struct Theme {
	pub accent_color: String,
	pub background_color: String,
	pub key: &'static str,
	pub text_color: String
}

impl Theme {
    pub fn dark() -> Theme {
        Theme {
			accent_color: String::from("#fbdf46"),
			background_color: String::from("#1a1a1a"),
			key: "dark",
			text_color: String::from("#c9c9c9")
		}
    }

    pub fn light() -> Theme {
        Theme {
			accent_color: String::from("#9f2e02"),
			background_color: String::from("#fafafa"),
			key: "light",
			text_color: String::from("#272727")
		}
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::light()
    }
}
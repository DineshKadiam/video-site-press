use indoc::formatdoc;

#[derive(Clone, Debug)]
pub struct Theme {
	pub key: &'static str,
	pub kind: ThemeKind
}

#[derive(Clone, Debug)]
pub enum ThemeKind {	
	Adaptive {
		dark: ThemeVars,
		light: ThemeVars
	},
	Static(ThemeVars)
}

#[derive(Clone, Debug)]
pub struct ThemeVars {
	pub accent_color: String,
	pub background_color: String,
	pub text_color: String
}

impl Theme {
	pub fn adaptive() -> Theme {
		Theme {
			key: "adaptive",
			kind: ThemeKind::Adaptive {
				dark: ThemeVars::dark(),
				light: ThemeVars::light()
			}
		}
	}

    pub fn dark() -> Theme {
		Theme {
			key: "dark",
			kind: ThemeKind::Static(ThemeVars::dark())
		}
    }

    pub fn from_key(key: &str) -> Result<Theme, String> {
    	match key {
    		"adaptive" => Ok(Theme::adaptive()),
    		"dark" => Ok(Theme::dark()),
    		"light" => Ok(Theme::light()),
    		_ => Err(format!("Unknown theme key '{key}' (known keys are 'adaptive', 'dark' and 'light')"))
        }
    }

    pub fn light() -> Theme {
		Theme {
			key: "light",
			kind: ThemeKind::Static(ThemeVars::light())
		}
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::light()
    }
}

impl ThemeVars {
    pub fn dark() -> ThemeVars {
        ThemeVars {
			accent_color: String::from("#fbdf46"),
			background_color: String::from("#1a1a1a"),
			text_color: String::from("#c9c9c9")
		}
    }

    pub fn light() -> ThemeVars {
        ThemeVars {
			accent_color: String::from("#9f2e02"),
			background_color: String::from("#fafafa"),
			text_color: String::from("#272727")
		}
    }

    pub fn print(&self) -> String {
    	let accent_color = &self.accent_color;
	    let background_color = &self.background_color;
	    let text_color = &self.text_color;

	    formatdoc!(r#"
            --c-accent: {accent_color};
            --c-background: {background_color};
            --c-text: {text_color};
	    "#)
    }
}
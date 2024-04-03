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

/// Theme color variables are specified using notions of distance:
/// In dark color schemes the background is dark, while the foreground
/// is bright, and the other way round in bright color schemes.
#[derive(Clone, Debug)]
pub struct ThemeVars {
	pub accent: String,
	/// Background 1 (furthest away/bottom layer)
	pub bg_1: String,
	/// Background 2 (further away)
	pub bg_2: String,
	/// Background 3 (far away)
	pub bg_3: String,
	/// Between background and middleground
	pub bg_mg: String,
	/// Foreground 1 (closest/top layer)
	pub fg_1: String,
	/// Foreground 2 (closer)
	pub fg_2: String,
	/// Foreground 3 (close)
	pub fg_3: String,
	/// Middleground  (in between)
	pub mg: String
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
			accent: String::from("#fbdf46"),
			bg_1: String::from("#161616"),
			bg_2: String::from("#262626"),
			bg_3: String::from("#333"),
			bg_mg: String::from("#444"),
			fg_1: String::from("#fff"),
			fg_2: String::from("#c9c9c9"),
			fg_3: String::from("#b2b2b2"),
			mg: String::from("#888")
		}
    }

    pub fn light() -> ThemeVars {
        ThemeVars {
			accent: String::from("#9f2e02"),
			bg_1: String::from("#fff"),
			bg_2: String::from("#eee"),
			bg_3: String::from("#ddd"),
			bg_mg: String::from("#ccc"),
			fg_1: String::from("#000"),
			fg_2: String::from("#272727"),
			fg_3: String::from("#444"),
			mg: String::from("#888")
		}
    }

    pub fn print(&self) -> String {
    	let accent = &self.accent;
	    let bg_1 = &self.bg_1;
	    let bg_2 = &self.bg_2;
	    let bg_3 = &self.bg_3;
	    let bg_mg = &self.bg_mg;
	    let fg_1 = &self.fg_1;
	    let fg_2 = &self.fg_2;
	    let fg_3 = &self.fg_3;
	    let mg = &self.mg;

	    formatdoc!(r#"
            --accent: {accent};
            --bg-1: {bg_1};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --mg: {mg};
	    "#)
    }
}
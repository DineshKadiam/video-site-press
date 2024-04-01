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
	pub color_accent: String,
	/// Background 1 (furthest away/bottom layer)
	pub color_bg_1: String,
	/// Background 2 (further away)
	pub color_bg_2: String,
	/// Background 3 (far away)
	pub color_bg_3: String,
	/// Between background and middleground
	pub color_bg_mg: String,
	/// Foreground 1 (closest/top layer)
	pub color_fg_1: String,
	/// Foreground 2 (closer)
	pub color_fg_2: String,
	/// Foreground 3 (close)
	pub color_fg_3: String,
	/// Middleground  (in between)
	pub color_mg: String
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
			color_accent: String::from("#fbdf46"),
			color_bg_1: String::from("#161616"),
			color_bg_2: String::from("#262626"),
			color_bg_3: String::from("#333"),
			color_bg_mg: String::from("#444"),
			color_fg_1: String::from("#fff"),
			color_fg_2: String::from("#c9c9c9"),
			color_fg_3: String::from("#b2b2b2"),
			color_mg: String::from("#888")
		}
    }

    pub fn light() -> ThemeVars {
        ThemeVars {
			color_accent: String::from("#9f2e02"),
			color_bg_1: String::from("#fff"),
			color_bg_2: String::from("#eee"),
			color_bg_3: String::from("#ddd"),
			color_bg_mg: String::from("#ccc"),
			color_fg_1: String::from("#000"),
			color_fg_2: String::from("#272727"),
			color_fg_3: String::from("#444"),
			color_mg: String::from("#888")
		}
    }

    pub fn print(&self) -> String {
    	let color_accent = &self.color_accent;
	    let color_bg_1 = &self.color_bg_1;
	    let color_bg_2 = &self.color_bg_2;
	    let color_bg_3 = &self.color_bg_3;
	    let color_bg_mg = &self.color_bg_mg;
	    let color_fg_1 = &self.color_fg_1;
	    let color_fg_2 = &self.color_fg_2;
	    let color_fg_3 = &self.color_fg_3;
	    let color_mg = &self.color_mg;

	    formatdoc!(r#"
            --c-accent: {color_accent};
            --c-bg-1: {color_bg_1};
            --c-bg-2: {color_bg_2};
            --c-bg-3: {color_bg_3};
            --c-bg-mg: {color_bg_mg};
            --c-fg-1: {color_fg_1};
            --c-fg-2: {color_fg_2};
            --c-fg-3: {color_fg_3};
            --c-mg: {color_mg};
	    "#)
    }
}
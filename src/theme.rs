use base64::Engine;
use base64::prelude::BASE64_STANDARD;
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
	pub bg_1_overlay: String,
	/// Background 2 (further away) with some transparency
	pub bg_2: String,
	/// Background 3 (far away)
	pub bg_3: String,
	/// Between background and middleground
	pub bg_mg: String,
	/// Indicate dangerous action, e.g. button background (practically speaking: dark-red)
	pub danger_bg: String,
	/// Indicate dangerous action, e.g. button foreground (practically speaking: white)
	pub danger_fg: String,
	/// Indicate dangerous action, e.g. button middleground/hover (practically speaking: bright-red)
	pub danger_mg: String,
	/// Foreground 1 (closest/top layer)
	pub fg_1: String,
	/// Foreground 2 (closer)
	pub fg_2: String,
	/// Foreground 3 (close)
	pub fg_3: String,
	/// Between foreground and middleground
	pub fg_mg: String,
	/// Middleground  (in between)
	pub mg: String
}

impl Theme {
	/// Splash gradient blend mode
	const SPLASH_BLEND: &'static str = "color";

	/// Splash background (furthest away/bottom layer)
	const SPLASH_BG: &'static str = "#9d1a57";

	/// Splash foreground (closest/top layer)
	const SPLASH_FG: &'static str = "#fff";

	/// Splash Northeast corner gradient color
	const SPLASH_NE: &'static str = "#43437d";

	/// Splash Northwest corner gradient color
	const SPLASH_NW: &'static str = "#cc0a2e";

	/// Splash Southeast corner gradient color
	const SPLASH_SE: &'static str = "#9b0000";

	/// Splash Southwest corner gradient color
	const SPLASH_SW: &'static str = "#f7dc00";

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

    pub fn print_splash_vars(&self) -> String {
	    let splash_blend = Theme::SPLASH_BLEND;
	    let splash_bg = Theme::SPLASH_BG;
	    let splash_fg = Theme::SPLASH_FG;
	    let splash_ne = Theme::SPLASH_NE;
	    let splash_nw = Theme::SPLASH_NW;
	    let splash_se = Theme::SPLASH_SE;
	    let splash_sw = Theme::SPLASH_SW;

	    formatdoc!(r#"
	    	body {{
	            --splash-blend: {splash_blend};
	            --splash-bg: {splash_bg};
	            --splash-fg: {splash_fg};
	            --splash-ne: {splash_ne};
	            --splash-nw: {splash_nw};
	            --splash-se: {splash_se};
	            --splash-sw: {splash_sw};
	        }}
	    "#)
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::adaptive()
    }
}

impl ThemeVars {
	fn checkerboard_base64(&self) -> String {
		let bg_2 = &self.bg_2;
		let bg_3 = &self.bg_3;

		let svg = formatdoc!(r#"
			<svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
	            <rect fill="{bg_2}" height="32" width="32" x="0" y="0"/>
	            <rect fill="{bg_2}" height="32" width="32" x="32" y="32"/>
	            <rect fill="{bg_3}" height="32" width="32" x="0" y="32"/>
	            <rect fill="{bg_3}" height="32" width="32" x="32" y="0"/>
	        </svg>
	    "#);

		BASE64_STANDARD.encode(svg)
	}

    pub fn dark() -> ThemeVars {
        ThemeVars {
			accent: String::from("#fbdf46"),
			bg_1: String::from("#161616"),
			bg_1_overlay: String::from("rgba(22, 22, 22, .85)"),
			bg_2: String::from("#262626"),
			bg_3: String::from("#333"),
			bg_mg: String::from("#444"),
			danger_bg: String::from("hsl(350, 55%, 40.5%)"),
			danger_fg: String::from("var(--fg-1)"),
			danger_mg: String::from("hsl(350, 55%, 50.5%)"),
			fg_1: String::from("#fff"),
			fg_2: String::from("#c9c9c9"),
			fg_3: String::from("#b2b2b2"),
			fg_mg: String::from("#989898"),
			mg: String::from("#888")
		}
    }

    pub fn light() -> ThemeVars {
        ThemeVars {
			accent: String::from("#9f2e02"),
			bg_1_overlay: String::from("rgba(255, 255, 255, .85)"),
			bg_1: String::from("#fff"),
			bg_2: String::from("#eee"),
			bg_3: String::from("#ddd"),
			bg_mg: String::from("#ccc"),
			danger_bg: String::from("hsl(350, 55%, 40.5%)"),
			danger_fg: String::from("var(--bg-1)"),
			danger_mg: String::from("hsl(350, 55%, 50.5%)"),
			fg_1: String::from("#000"),
			fg_2: String::from("#272727"),
			fg_3: String::from("#444"),
			fg_mg: String::from("#666"),
			mg: String::from("#888")
		}
    }

    pub fn print(&self) -> String {
    	let accent = &self.accent;
	    let bg_1 = &self.bg_1;
	    let bg_1_overlay = &self.bg_1_overlay;
	    let bg_2 = &self.bg_2;
	    let bg_3 = &self.bg_3;
	    let bg_mg = &self.bg_mg;
	    let checkerboard_base64 = self.checkerboard_base64();
	    let danger_bg = &self.danger_bg;
	    let danger_fg = &self.danger_fg;
	    let danger_mg = &self.danger_mg;
	    let fg_1 = &self.fg_1;
	    let fg_2 = &self.fg_2;
	    let fg_3 = &self.fg_3;
	    let fg_mg = &self.fg_mg;
	    let mg = &self.mg;

	    formatdoc!(r#"
            --accent: {accent};
            --bg-1: {bg_1};
            --bg-1-overlay: {bg_1_overlay};
            --bg-2: {bg_2};
            --bg-3: {bg_3};
            --bg-mg: {bg_mg};
            --checkerboard: url("data:image/svg+xml;base64,{checkerboard_base64}");
            --danger-bg: {danger_bg};
            --danger-fg: {danger_fg};
            --danger-mg: {danger_mg};
            --fg-1: {fg_1};
            --fg-2: {fg_2};
            --fg-3: {fg_3};
            --fg-mg: {fg_mg};
            --mg: {mg};
	    "#)
    }
}
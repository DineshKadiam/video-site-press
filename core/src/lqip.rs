// SPDX-FileCopyrightText: 2025-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_derive::{Deserialize, Serialize};

/// Low Quality Image Preview
/// An image reduced down to a 2x2 matrix (4 pixels). Each corner
/// (names indicate compass directions) stores the computed averaged
/// RGB color for that area in the image.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lqip {
    pub ne: (u8, u8, u8),
    pub nw: (u8, u8, u8),
    pub se: (u8, u8, u8),
    pub sw: (u8, u8, u8)
}

impl Lqip {
    // We currently only use the radial gradient implementation, but
    // keeping this one around in case we want to revisit it.
    // pub fn gradient_linear(&self) -> String {
    //     // 0deg is a gradient from bottom to top
    //     let ne = format!("linear-gradient(-135deg, {}, transparent)", Lqip::rgb(self.ne));
    //     let nw = format!("linear-gradient(135deg, {}, transparent)", Lqip::rgb(self.nw));
    //     let se = format!("linear-gradient(-45deg, {}, transparent)", Lqip::rgb(self.se));
    //     let sw = format!("linear-gradient(45deg, {}, transparent)", Lqip::rgb(self.sw));

    //     format!("background: {ne}, {nw}, {se}, {sw};")
    // }

    pub fn background_css(&self) -> String {
        let ne = format!("radial-gradient(ellipse at top right, {}, transparent)", Lqip::rgb(self.ne));
        let nw = format!("radial-gradient(ellipse at top left, {}, transparent)", Lqip::rgb(self.nw));
        let se = format!("radial-gradient(ellipse at bottom right, {}, transparent)", Lqip::rgb(self.se));
        let sw = format!("radial-gradient(ellipse at bottom left, {}, transparent)", Lqip::rgb(self.sw));

        format!("background: {ne}, {nw}, {se}, {sw};")
    }

    pub fn js_array(&self) -> String {
        let ne = format!("[{},{},{}]", self.ne.0, self.ne.1, self.ne.2);
        let nw = format!("[{},{},{}]", self.nw.0, self.nw.1, self.nw.2);
        let se = format!("[{},{},{}]", self.se.0, self.se.1, self.se.2);
        let sw = format!("[{},{},{}]", self.sw.0, self.sw.1, self.sw.2);

        format!("[{ne},{nw},{se},{sw}]")
    }

    pub fn new() -> Lqip {
        Lqip {
            ne: (0, 0, 0),
            nw: (0, 0, 0),
            se: (0, 0, 0),
            sw: (0, 0, 0)
        }
    }

    pub fn ne_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.ne.0, self.ne.1, self.ne.2)
    }

    pub fn nw_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.nw.0, self.nw.1, self.nw.2)
    }

    fn rgb(corner: (u8, u8, u8)) -> String {
        format!("rgb({}, {}, {})", corner.0, corner.1, corner.2)
    }

    pub fn se_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.se.0, self.se.1, self.se.2)
    }

    pub fn sw_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.sw.0, self.sw.1, self.sw.2)
    }
}

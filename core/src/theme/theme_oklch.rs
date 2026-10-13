// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use indoc::formatdoc;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct OkaPercent(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct OkcPercent(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct OkhDegrees(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct OklPercent(pub f32);

#[derive(Clone, Debug)]
pub struct Oklch {
    pub l: OklPercent,
    pub c: OkcPercent,
    pub h: OkhDegrees,
    pub a: Option<OkaPercent>
}

/// Theme color variables are specified using notions of distance:
/// In dark color schemes the background is dark, while the foreground
/// is bright, and the other way round in light color schemes.
/// Some things are constantly bright or dark in both dark and light
/// color schemes, these use aliases with dark/light terminology.
#[derive(Clone, Debug)]
pub struct ThemeOklch {
    pub acc: Oklch,
    pub b1: Oklch,
    pub b2: Oklch,
    pub b3: Oklch,
    pub bm: Oklch,
    pub d1: Oklch,
    pub d2: Oklch,
    pub d3: Oklch,
    pub dm: Oklch,
    pub f1: Oklch,
    pub f2: Oklch,
    pub f3: Oklch,
    pub fm: Oklch,
    pub l1: Oklch,
    pub l2: Oklch,
    pub l3: Oklch,
    pub m: Oklch
}

impl Display for OkaPercent {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Display for OkcPercent {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Display for OkhDegrees {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Display for OklPercent {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Oklch {
    pub fn alpha(&self, a: OkaPercent) -> Oklch {
        Oklch {
            l: self.l,
            c: self.c,
            h: self.h,
            a: Some(a)
        }
    }

    pub fn chroma(&self, c: OkcPercent) -> Oklch {
        Oklch {
            l: self.l,
            c,
            h: self.h,
            a: self.a
        }
    }

    pub fn css(&self) -> String {
        let l = &self.l;
        let c = &self.c;
        let h = &self.h;

        match &self.a {
            Some(a) => format!("oklch({l}% {c}% {h} / {a}%)"),
            None => format!("oklch({l}% {c}% {h})")
        }
    }

    pub fn hue(&self, h: OkhDegrees) -> Oklch {
        Oklch {
            l: self.l,
            c: self.c,
            h,
            a: self.a
        }
    }

    pub fn lch(
        l: OklPercent,
        c: OkcPercent,
        h: OkhDegrees
    ) -> Oklch {
        Oklch {
            l,
            c,
            h,
            a: None
        }
    }

    pub const fn lch_raw(
        l: f32,
        c: f32,
        h: f32
    ) -> Oklch {
        Oklch {
            l: OklPercent(l),
            c: OkcPercent(c),
            h: OkhDegrees(h),
            a: None
        }
    }

    pub fn lcha(
        l: OklPercent,
        c: OkcPercent,
        h: OkhDegrees,
        a: OkaPercent
    ) -> Oklch {
        Oklch {
            l,
            c,
            h,
            a: Some(a)
        }
    }
}

impl ThemeOklch {
    fn checkerboard_base64(&self) -> String {
        let b2 = &self.b2.css();
        let b3 = &self.b3.css();

        let svg = formatdoc!(r#"
            <svg width="64" height="64" version="1.1" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
                <rect fill="{b2}" height="32" width="32" x="0" y="0"/>
                <rect fill="{b2}" height="32" width="32" x="32" y="32"/>
                <rect fill="{b3}" height="32" width="32" x="0" y="32"/>
                <rect fill="{b3}" height="32" width="32" x="32" y="0"/>
            </svg>
        "#);

        BASE64_STANDARD.encode(svg)
    }

    pub fn css(&self, prefix: &str) -> String {
        let mut declarations = Vec::new();

        let mut add = |name, value| {
            let css = format!("--{prefix}{name}:{value};");
            declarations.push(css);
        };

        let checkerboard_svg = self.checkerboard_base64();
        let checkerboard_url = format!(r#"url("data:image/svg+xml;base64,{checkerboard_svg}")"#);

        add("acc", self.acc.css());
        add("b1", self.b1.css());
        add("b1-85", self.b1.alpha(OkaPercent(85.0)).css());
        add("b1-92", self.b1.alpha(OkaPercent(92.0)).css());
        add("b2", self.b2.css());
        add("b3", self.b3.css());
        add("bm", self.bm.css());
        // TODO: Technically editor only, move over into that context sometime
        add("checkerboard", checkerboard_url);
        add("d1", self.d1.css());
        add("d1-15", self.d1.alpha(OkaPercent(15.0)).css());
        add("d1-70-mono", self.d1.alpha(OkaPercent(70.0)).chroma(OkcPercent(0.0)).css());
        add("d1-85-mono", self.d1.alpha(OkaPercent(85.0)).chroma(OkcPercent(0.0)).css());
        add("d1-mono", self.d1.chroma(OkcPercent(0.0)).css());
        add("d2", self.d2.css());
        add("d3", self.d3.css());
        add("d3-mono", self.d3.chroma(OkcPercent(0.0)).css());
        add("dm", self.dm.css());
        add("f1", self.f1.css());
        add("f1-5", self.f1.alpha(OkaPercent(5.0)).css());
        add("f2", self.f2.css());
        add("f3", self.f3.css());
        add("fm", self.fm.css());
        add("l1-mono", self.l1.chroma(OkcPercent(0.0)).css());
        add("l1-10-mono", self.l1.alpha(OkaPercent(10.0)).chroma(OkcPercent(0.0)).css());
        add("l2-mono", self.l2.chroma(OkcPercent(0.0)).css());
        add("l3-mono", self.l3.chroma(OkcPercent(0.0)).css());
        add("m", self.m.css());

        declarations.join("")
    }

    pub fn preview_gradient_vars(&self, prefix: &str) -> String {
        let a_oklch = self.b3.css();
        let b_oklch = self.bm.css();

        format!("--{prefix}a:{a_oklch};--{prefix}b:{b_oklch};")
    }
}

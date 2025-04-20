// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

/// We define/store:
/// - lightness and saturation as 0.0-1.0
/// - chroma as 0.0-infinity (sensible range defined by OKLCH color model)
/// - hue as 0-360
#[derive(Clone, Debug)]
pub enum Color {
    Chromatic {
        hsl: (u16, f32, f32),
        oklch: (f32, f32, u16)
    },
    Gradient {
        begin_hsl: (u16, f32, f32),
        begin_oklch: (f32, f32, u16),
        end_hsl: (u16, f32, f32),
        end_oklch: (f32, f32, u16)
    },
    Gray {
        hsl_l: f32,
        oklch_l: f32
    }
}

impl Color {
    pub fn hsl(&self) -> String {
        match self {
            Color::Chromatic { hsl, .. } => {
                let h = hsl.0;
                let s = hsl.1 * 100.0;
                let l = hsl.2 * 100.0;

                format!("hsl({h} {s}% {l}%)")
            }
            Color::Gradient { begin_hsl, end_hsl, .. } => {
                let begin_h = begin_hsl.0;
                let begin_s = begin_hsl.1 * 100.0;
                let begin_l = begin_hsl.2 * 100.0;
                let end_h = end_hsl.0;
                let end_s = end_hsl.1 * 100.0;
                let end_l = end_hsl.2 * 100.0;

                format!("linear-gradient(180deg, hsl({begin_h} {begin_s}% {begin_l}%), hsl({end_h} {end_s}% {end_l}%))")
            }
            Color::Gray { hsl_l, .. } => {
                let l = hsl_l * 100.0;
                format!("hsl(0 0% {l}%)")
            }
        }
    }

    /// Pass alpha in percent from 0.0-100.0%
    pub fn hsla(&self, a: f32) -> String {
        match self {
            Color::Chromatic { hsl, .. } => {
                let h = hsl.0;
                let s = hsl.1 * 100.0;
                let l = hsl.2 * 100.0;

                format!("hsl({h} {s}% {l}% / {a}%)")
            }
            Color::Gradient { begin_hsl, end_hsl, .. } => {
                let begin_h = begin_hsl.0;
                let begin_s = begin_hsl.1 * 100.0;
                let begin_l = begin_hsl.2 * 100.0;
                let end_h = end_hsl.0;
                let end_s = end_hsl.1 * 100.0;
                let end_l = end_hsl.2 * 100.0;

                format!("linear-gradient(180deg, hsl({begin_h} {begin_s}% {begin_l}% / {a}%), hsl({end_h} {end_s}% {end_l}% / {a}%))")
            }
            Color::Gray { hsl_l, .. } => {
                let l = hsl_l * 100.0;
                format!("hsl(0 0% {l}% / {a}%)")
            }
        }
    }

    pub fn oklch(&self) -> String {
        match self {
            Color::Chromatic { oklch, .. } => {
                let l = oklch.0 * 100.0;
                let c = oklch.1;
                let h = oklch.2;

                format!("oklch({l}% {c} {h})")
            }
            Color::Gradient { begin_oklch, end_oklch, .. } => {
                let begin_l = begin_oklch.0 * 100.0;
                let begin_c = begin_oklch.1;
                let begin_h = begin_oklch.2;
                let end_l = end_oklch.0 * 100.0;
                let end_c = end_oklch.1;
                let end_h = end_oklch.2;

                format!("linear-gradient(180deg, oklch({begin_l}% {begin_c} {begin_h}), oklch({end_l}% {end_c} {end_h}))")
            }
            Color::Gray { oklch_l, .. } => {
                let l = oklch_l * 100.0;
                format!("oklch({l}% 0 0)")
            }
        }
    }

    /// Pass alpha in percent from 0.0-100.0%
    pub fn oklcha(&self, a: f32) -> String {
        match self {
            Color::Chromatic { oklch, .. } => {
                let l = oklch.0 * 100.0;
                let c = oklch.1;
                let h = oklch.2;

                format!("oklch({l}% {c} {h} / {a}%)")
            }
            Color::Gradient { begin_oklch, end_oklch, .. } => {
                let begin_l = begin_oklch.0 * 100.0;
                let begin_c = begin_oklch.1;
                let begin_h = begin_oklch.2;
                let end_l = end_oklch.0 * 100.0;
                let end_c = end_oklch.1;
                let end_h = end_oklch.2;

                format!("linear-gradient(180deg, oklch({begin_l}% {begin_c} {begin_h} / {a}%), oklch({end_l}% {end_c} {end_h} / {a}%))")
            }
            Color::Gray { oklch_l, .. } => {
                let l = oklch_l * 100.0;
                format!("oklch({l}% 0 0 / {a})")
            }
        }
    }
}

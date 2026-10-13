// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Display;
use std::ops::Range;

use crate::{
    OkcPercent,
    OkhDegrees,
    OklPercent,
    Oklch,
    ThemeOklch,
    ThemePaint
};

/// This is a "design as data" kind of representation that stores the
/// fundamental lightness and alpha relationships in our dark and light
/// themes. The parameters in this struct are later combined with the
/// user-supplied theme settings (chroma and hue mostly) to make up the final
/// colors of the theme.
///
/// Glossary:
/// - b = back/background
/// - f = front/foreground
/// - m = middle/middleground
/// - a = alpha
/// - l = lightness
/// - acc = accent
pub struct ThemeParams {
    pub b1_l_range: Range<f32>,
    pub b2_l_range: Range<f32>,
    pub b3_l_range: Range<f32>,
    pub bm_l: f32,
    pub d1_l_range: Range<f32>,
    pub d2_l_range: Range<f32>,
    pub d3_l_range: Range<f32>,
    pub dark: bool,
    pub dm_l: f32,
    pub f1_l_range: Range<f32>,
    pub f2_l_range: Range<f32>,
    pub f3_l_range: Range<f32>,
    pub fm_l: f32,
    pub l1_l_range: Range<f32>,
    pub l2_l_range: Range<f32>,
    pub l3_l_range: Range<f32>,
    pub m_l: f32
}

/// This function returns the limited maximum `contrast` that can be used to
/// interpolate lightness values from the theme param ranges
fn max_contrast_for_chroma(
    chroma: f32,
    dark_theme: bool,
    furthest_background_range: &Range<f32>
) -> f32 {
    // Noticable erratic shifts in the OKLCH color model appear for very dark
    // values with around 40%+ chroma, so we explicitly compensate for this
    if dark_theme && chroma > 40.0 {
        let compensation_factor = (chroma - 40.0) / 60.0;

        let darkest_range_brightest = furthest_background_range.start;
        let darkest_range_darkest = furthest_background_range.end;

        // Around <=20% lightness oklch produces very erratic hue shifts at
        // extreme chroma levels, therefore we shift lightness quite a bit
        // away from pure black when we're dealing with extreme chroma
        // levels
        let min_lightness = compensation_factor * 20.0;

        if darkest_range_darkest >= min_lightness {
            // The full range can be used
            return 100.0;
        } else if darkest_range_brightest < min_lightness {
            // This means that the brightest lightness produced by the theme
            // at 0% contrast still doesn't suffice to compensate (raise
            // lightness enough to avoid chroma artifacts). Ideally the theme
            // should be set up to not cross this boundary, but minimal
            // overshoot is ok in practice.
            return 0.0;
        } else {
            let range_lightness_delta = darkest_range_brightest - darkest_range_darkest;
            let max_contrast = (1.0 - (min_lightness - darkest_range_darkest) / range_lightness_delta) * 100.0;

            return max_contrast;
        }
    }

    // OKLCH compensates conflicting color components (e.g. 100% lightness
    // with 100% chroma) very sensibly in the bright range, therefore we
    // don't apply any bespoke compensation for light colors

    return 100.0;
}

impl ThemeParams {
    const DARK_D1_L_RANGE: Range<f32> = 20.0..0.0;
    const DARK_D2_L_RANGE: Range<f32> = 26.9..20.0;
    const DARK_D3_L_RANGE: Range<f32> = 32.0..28.0;
    const DARK_DM: f32 = 38.7;
    const DARK_L1_L_RANGE: Range<f32> = 100.0..100.0;
    const DARK_L2_L_RANGE: Range<f32> = 84.0..84.0;
    const DARK_L3_L_RANGE: Range<f32> = 77.0..77.0;

    pub const DARK: ThemeParams = ThemeParams {
        b1_l_range: Self::DARK_D1_L_RANGE,
        b2_l_range: Self::DARK_D2_L_RANGE,
        b3_l_range: Self::DARK_D3_L_RANGE,
        bm_l: Self::DARK_DM,
        d1_l_range: Self::DARK_D1_L_RANGE,
        d2_l_range: Self::DARK_D2_L_RANGE,
        d3_l_range: Self::DARK_D3_L_RANGE,
        dark: true,
        dm_l: Self::DARK_DM,
        f1_l_range: Self::DARK_L1_L_RANGE,
        f2_l_range: Self::DARK_L2_L_RANGE,
        f3_l_range: Self::DARK_L3_L_RANGE,
        fm_l: 68.0,
        l1_l_range: Self::DARK_L1_L_RANGE,
        l2_l_range: Self::DARK_L2_L_RANGE,
        l3_l_range: Self::DARK_L3_L_RANGE,
        m_l: 62.7
    };

    const LIGHT_B1_L_RANGE: Range<f32> = 92.0..100.0;
    const LIGHT_B2_L_RANGE: Range<f32> = 85.0..91.0;
    const LIGHT_B3_L_RANGE: Range<f32> = 78.0..81.0;
    const LIGHT_D1_L_RANGE: Range<f32> = 0.0..0.0;
    const LIGHT_D2_L_RANGE: Range<f32> = 27.2..27.2;
    const LIGHT_D3_L_RANGE: Range<f32> = 38.6..38.6;
    const LIGHT_DM: f32 = 51.0;

    // The light 1-3 tones – distinctly from the background 1-3 tones – are
    // used as alternate foreground tones over the dark 1-3 tones, hence we
    // separately track them and let them reach full brightness regardless of
    // dynamic range. Consider for instance the background (b1) being a
    // bright gray, while there is pure white text (l1) inside the player,
    // appearing on some dark tone (d1-3).
    const LIGHT_L1_L_RANGE: Range<f32> = 100.0..100.0;
    const LIGHT_L2_L_RANGE: Range<f32> = 95.0..95.0;
    const LIGHT_L3_L_RANGE: Range<f32> = 90.0..90.0;

    pub const LIGHT: ThemeParams = ThemeParams {
        b1_l_range: Self::LIGHT_B1_L_RANGE,
        b2_l_range: Self::LIGHT_B2_L_RANGE,
        b3_l_range: Self::LIGHT_B3_L_RANGE,
        bm_l: 72.0,
        d1_l_range: Self::LIGHT_D1_L_RANGE,
        d2_l_range: Self::LIGHT_D2_L_RANGE,
        d3_l_range: Self::LIGHT_D3_L_RANGE,
        dark: false,
        dm_l: Self::LIGHT_DM,
        f1_l_range: Self::LIGHT_D1_L_RANGE,
        f2_l_range: Self::LIGHT_D2_L_RANGE,
        f3_l_range: Self::LIGHT_D3_L_RANGE,
        fm_l: Self::LIGHT_DM,
        l1_l_range: Self::LIGHT_L1_L_RANGE,
        l2_l_range: Self::LIGHT_L2_L_RANGE,
        l3_l_range: Self::LIGHT_L3_L_RANGE,
        m_l: 62.7
    };

    pub fn compute_theme(&self, paint: &Option<ThemePaint>) -> ThemeOklch {
        let paint = paint
            .clone()
            .unwrap_or_else(|| ThemePaint::default());

        const BASE_CHROMA_DAMPENING: f32 = 0.4;
        const LAYERED_CHROMA_DAMPENING: f32 = 0.4;

        let accent_brightening = paint.accent_brightening();
        let accent_chroma = paint.accent_chroma();
        let accent_hue = paint.accent_hue();
        let base_chroma = paint.base_chroma() as f32 * BASE_CHROMA_DAMPENING;
        let base_hue = paint.base_hue();
        let contrast = paint.contrast();
        let layered_chroma = paint.layered_chroma() as f32 * LAYERED_CHROMA_DAMPENING;

        let pick_from_range = ThemeParams::pick_from_range;

        let max_contrast = max_contrast_for_chroma(base_chroma, self.dark, &self.b1_l_range);
        let limited_contrast = (contrast as f32 / 100.0) * max_contrast;

        let acc_l = OklPercent(70.0 + (accent_brightening as f32 / 100.0) * 30.0);
        let b1_l = OklPercent(pick_from_range(limited_contrast, &self.b1_l_range));
        let b2_l = OklPercent(pick_from_range(limited_contrast, &self.b2_l_range));
        let b3_l = OklPercent(pick_from_range(limited_contrast, &self.b3_l_range));
        let bm_l = OklPercent(self.bm_l);
        let d1_l = OklPercent(pick_from_range(limited_contrast, &self.d1_l_range));
        let d2_l = OklPercent(pick_from_range(limited_contrast, &self.d2_l_range));
        let d3_l = OklPercent(pick_from_range(limited_contrast, &self.d3_l_range));
        let dm_l = OklPercent(self.dm_l);
        let f1_l = OklPercent(pick_from_range(limited_contrast, &self.f1_l_range));
        let f2_l = OklPercent(pick_from_range(limited_contrast, &self.f2_l_range));
        let f3_l = OklPercent(pick_from_range(limited_contrast, &self.f3_l_range));
        let fm_l = OklPercent(self.fm_l);
        let l1_l = OklPercent(pick_from_range(limited_contrast, &self.l1_l_range));
        let l2_l = OklPercent(pick_from_range(limited_contrast, &self.l2_l_range));
        let l3_l = OklPercent(pick_from_range(limited_contrast, &self.l3_l_range));
        let m_l = OklPercent(self.m_l);

        // We observed that in the light theme the base chroma needs to bleed more
        // into the detail chroma than in the dark theme in order to be perceived
        // as equally saturated.
        let base_chroma_bleed = if self.dark { 0.5 } else { 0.75 };

        // The final layered chroma is at least the amount of chroma bleeding over
        // from base_chroma, or higher if the user requested a higher
        // layeredChroma expliclity.
        let base_plus_layered_chroma = (base_chroma * base_chroma_bleed).max(layered_chroma);

        const HIGH_CHROMA: f32 = 0.5;
        const LOW_CHROMA: f32 = 0.03;
        const LOWEST_CHROMA: f32 = 0.0;

        // This function shapes chroma distribution across the lower
        // tones (b3-f1) by attenuating the chroma the closer the passed
        // lightness lies to the foreground lightness. For this it uses fixed
        // ratios.
        let chroma_shaper = |lightness: OklPercent| {
            let b3_l = b3_l.0;
            let f1_l = f1_l.0;
            let f3_l = f3_l.0;
            let lightness = lightness.0;

            if self.dark {
                if lightness <= f3_l {
                    HIGH_CHROMA - (HIGH_CHROMA - LOW_CHROMA) * ((lightness - b3_l) / (f3_l - b3_l))
                } else if lightness <= f1_l {
                    LOW_CHROMA - (LOW_CHROMA - LOWEST_CHROMA) * ((lightness - f3_l) / (f1_l - f3_l))
                } else /* if lightness > f1_l */ {
                    LOWEST_CHROMA
                }
            } else {
                if lightness >= f3_l {
                    LOW_CHROMA + (HIGH_CHROMA - LOW_CHROMA) * ((lightness - f3_l) / (b3_l - f3_l))
                } else if lightness >= f1_l {
                    LOWEST_CHROMA + (LOW_CHROMA - LOWEST_CHROMA) * ((lightness - f1_l) / (f3_l - f1_l))
                } else /* if lightness < f1_l */ {
                    LOWEST_CHROMA
                }
            }
        };

        let acc_c = OkcPercent(accent_chroma as f32);
        let b1_c = OkcPercent(base_chroma);
        let b2_c = OkcPercent(base_plus_layered_chroma);
        let b3_c = OkcPercent(base_plus_layered_chroma);
        let bm_c = OkcPercent(base_chroma * chroma_shaper(bm_l));
        let d1_c = OkcPercent(base_chroma * chroma_shaper(d1_l));
        let d2_c = OkcPercent(base_chroma * chroma_shaper(d2_l));
        let d3_c = OkcPercent(base_chroma * chroma_shaper(d3_l));
        let dm_c = OkcPercent(base_chroma * chroma_shaper(dm_l));
        let f1_c = OkcPercent(base_chroma * chroma_shaper(f1_l));
        let f2_c = OkcPercent(base_chroma * chroma_shaper(f2_l));
        let f3_c = OkcPercent(base_chroma * chroma_shaper(f3_l));
        let fm_c = OkcPercent(base_chroma * chroma_shaper(fm_l));
        let l1_c = OkcPercent(base_chroma * chroma_shaper(l1_l));
        let l2_c = OkcPercent(base_chroma * chroma_shaper(l2_l));
        let l3_c = OkcPercent(base_chroma * chroma_shaper(l3_l));
        let m_c = OkcPercent(base_chroma * chroma_shaper(m_l));

        let acc_h = OkhDegrees(accent_hue as f32);
        let b1_h = OkhDegrees(base_hue as f32);
        let b2_h = OkhDegrees(base_hue as f32);
        let b3_h = OkhDegrees(base_hue as f32);
        let bm_h = OkhDegrees(base_hue as f32);
        let d1_h = OkhDegrees(base_hue as f32);
        let d2_h = OkhDegrees(base_hue as f32);
        let d3_h = OkhDegrees(base_hue as f32);
        let dm_h = OkhDegrees(base_hue as f32);
        let f1_h = OkhDegrees(base_hue as f32);
        let f2_h = OkhDegrees(base_hue as f32);
        let f3_h = OkhDegrees(base_hue as f32);
        let fm_h = OkhDegrees(base_hue as f32);
        let l1_h = OkhDegrees(base_hue as f32);
        let l2_h = OkhDegrees(base_hue as f32);
        let l3_h = OkhDegrees(base_hue as f32);
        let m_h = OkhDegrees(base_hue as f32);

        let acc = Oklch::lch(acc_l, acc_c, acc_h);
        let b1 = Oklch::lch(b1_l, b1_c, b1_h);
        let b2 = Oklch::lch(b2_l, b2_c, b2_h);
        let b3 = Oklch::lch(b3_l, b3_c, b3_h);
        let bm = Oklch::lch(bm_l, bm_c, bm_h);
        let d1 = Oklch::lch(d1_l, d1_c, d1_h);
        let d2 = Oklch::lch(d2_l, d2_c, d2_h);
        let d3 = Oklch::lch(d3_l, d3_c, d3_h);
        let dm = Oklch::lch(dm_l, dm_c, dm_h);
        let f1 = Oklch::lch(f1_l, f1_c, f1_h);
        let f2 = Oklch::lch(f2_l, f2_c, f2_h);
        let f3 = Oklch::lch(f3_l, f3_c, f3_h);
        let fm = Oklch::lch(fm_l, fm_c, fm_h);
        let l1 = Oklch::lch(l1_l, l1_c, l1_h);
        let l2 = Oklch::lch(l2_l, l2_c, l2_h);
        let l3 = Oklch::lch(l3_l, l3_c, l3_h);
        let m = Oklch::lch(m_l, m_c, m_h);

        ThemeOklch {
            acc,
            b1,
            b2,
            b3,
            bm,
            d1,
            d2,
            d3,
            dm,
            f1,
            f2,
            f3,
            fm,
            l1,
            l2,
            l3,
            m
        }
    }

    /// Outputs the params in js format, e.g. something like: "{fMl:45.0,f1L:0.0,...}"
    pub fn js(&self) -> String {
        let mut props = Vec::new();

        fn add(
            name: &str,
            props: &mut Vec<String>,
            value: impl Display
        ) {
            let js = format!("{name}:{value}");
            props.push(js);
        }

        fn range(range: &Range<f32>) -> String {
            let end = &range.end;
            let start = &range.start;

            format!("[{start},{end}]")
        }

        add("b1LRange", &mut props, range(&self.b1_l_range));
        add("b2LRange", &mut props, range(&self.b2_l_range));
        add("b3LRange", &mut props, range(&self.b3_l_range));
        add("bmL", &mut props, &self.bm_l);
        add("d1LRange", &mut props, range(&self.d1_l_range));
        add("d2LRange", &mut props, range(&self.d2_l_range));
        add("d3LRange", &mut props, range(&self.d3_l_range));
        add("dmL", &mut props, &self.dm_l);
        add("f1LRange", &mut props, range(&self.f1_l_range));
        add("f2LRange", &mut props, range(&self.f2_l_range));
        add("f3LRange", &mut props, range(&self.f3_l_range));
        add("fmL", &mut props, &self.fm_l);
        add("l1LRange", &mut props, range(&self.l1_l_range));
        add("l2LRange", &mut props, range(&self.l2_l_range));
        add("l3LRange", &mut props, range(&self.l3_l_range));
        add("mL", &mut props, &self.m_l);

        let props = props.join(",");

        format!("{{{props}}}")
    }

    /// Given a range between two float numbers (e.g. 0.0-42.0) and a percent
    /// value (e.g. 50%) this returns the float number on the range at "X%
    /// from A to B" (e.g. 21.0).
    fn pick_from_range(percent: f32, variable: &Range<f32>) -> f32 {
        variable.start + (percent / 100.0) * (variable.end - variable.start)
    }
}
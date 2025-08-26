// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

const SPECIAL_ASPECT_RATIOS: [(f32, &str); 10] = [
    (2.37, "64:27"),
    (1.78, "16:9"),
    (1.66, "5:3"),
    (1.5, "3:2"),
    (1.33, "4:3"),
    (0.75, "3:4"),
    (0.67, "2:3"),
    (0.6, "3:5"),
    (0.56, "9:16"),
    (0.42, "27:64")
];

#[derive(Clone, Debug)]
pub enum AspectRatio {
    /// Representation stores the user-supplied input (e.g. 16:9) while value stores
    /// the numeric result of dividing width parts by height parts (e.g. 1.78).
    Custom {
        representation: String,
        value: f32
    },
    // 16:9
    Default
}

impl AspectRatio {
    /// For some common, handpicked aspect ratios we try to automatically display
    /// them to the user (e.g. 16:9 instead of 1.7777778 or such).
    pub fn approximate_representation(numeric_ratio: f32) -> String {
        // Could be adjusted up/down if we feel our approximations are too
        // strictly/loosely determined.
        const APPROXIMATION_TOLERANCE: f32 = 0.01;

        for ratio in SPECIAL_ASPECT_RATIOS {
            if (ratio.0 - numeric_ratio).abs() <= APPROXIMATION_TOLERANCE {
                return ratio.1.to_string();
            }
        }

        format!("{numeric_ratio:.2}")
    }

    /// Parse a representation such as '1.78', '16:9' or '16/9' into an
    /// [AspectRatio] struct or return an error if the format is
    /// unsupported.
    pub fn from_representation(representation: &str) -> Result<AspectRatio, String> {
        if let Ok(ratio) = representation.parse::<f32>() {
            let aspect_ratio = AspectRatio::Custom {
                representation: representation.to_owned(),
                value: ratio
            };

            return Ok(aspect_ratio);
        }

        let mut parts = representation.split(['/', ':']);
        if let (Some(dividend), Some(divisor), None) = (parts.next(), parts.next(), parts.next()) {
            if let (Ok(dividend), Ok(divisor)) = (dividend.parse::<f32>(), divisor.parse::<f32>()) {
                let aspect_ratio = AspectRatio::Custom {
                    representation: representation.to_owned(),
                    value: dividend / divisor
                };

                return Ok(aspect_ratio);
            }
        }

        let error = format!("'{representation}' is not understood, allowed aspect formats are e.g. '1.78', '16:9' or '16/9')");
        Err(error)
    }

    /// Return the raw aspect ratio number ("width parts divided by height parts").
    pub fn numeric(&self) -> f32 {
        match self {
            AspectRatio::Default => 16.0/9.0,
            AspectRatio::Custom { value, .. } => *value
        }
    }

    /// Return the aspect ratio as it was supplied by the user (i.e. in a
    /// usually more meaningful/understandable representation such as 16:9
    /// instead of 1.78).
    pub fn representation(&self) -> &str {
        match self {
            AspectRatio::Default => "16:9",
            AspectRatio::Custom { representation, .. } => representation
        }
    }
}

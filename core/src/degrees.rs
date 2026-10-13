// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Encapsulates and abstracts degree values from 0-360 degrees. An instance
/// of Degrees can be parsed from and output back to a string with either no
/// unit ("90"), a symbolic unit ("90°") or a written out unit ("90deg").
#[derive(Clone, Copy, Debug)]
pub struct Degrees {
    /// Degree values can be supplied by the user without unit, with a
    /// symbolic unit (°) or a written out unit (deg), which we need to
    /// memorize for persisting it back to a manifest
    unit: Option<&'static str>,
    value: u16,
}

impl Degrees {
    pub fn new(unit: Option<&'static str>, value: u16) -> Degrees {
        Degrees {
            unit,
            value
        }
    }

    /// Returns the underlying degrees value
    pub fn value(&self) -> u16 {
        self.value
    }
}

impl Into<String> for &Degrees {
    fn into(self) -> String {
        let Degrees { unit, value } = self;

        if let Some(unit) = unit {
            format!("{value}{unit}")
        } else {
            value.to_string()
        }
    }
}

impl TryFrom<&str> for Degrees {
    type Error = ();

    fn try_from(degrees_str: &str) -> Result<Degrees, ()> {
        let parse = |signed, value_str: &str| {
            match value_str.parse::<u16>() {
                Ok(value) => {
                    if value <= 360 {
                        Ok(Degrees::new(signed, value))
                    } else {
                        Err(())
                    }
                }
                Err(_) => Err(())
            }
        };

        match degrees_str.strip_suffix('°') {
            Some(leading_str) => parse(Some("°"), leading_str),
            None => {
                match degrees_str.strip_suffix("deg") {
                    Some(leading_str) => parse(Some("deg"), leading_str),
                    None => {
                        parse(None, degrees_str)
                    }
                }
            }
        }
    }
}

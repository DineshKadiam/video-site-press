// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Encapsulates and abstracts percent values from 0-100(%)
#[derive(Clone, Copy, Debug)]
pub struct Percent {
    /// Percent values can be supplied by the user with or without % sign,
    /// which we need to memorize for persisting it back to a manifest
    signed: bool,
    value: u8
}

impl Percent {
    pub fn new(signed: bool, value: u8) -> Percent {
        Percent {
            signed,
            value
        }
    }

    /// Returns the underlying percent value
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Into<String> for &Percent {
    fn into(self) -> String {
        let Percent { signed, value } = self;

        if *signed {
            format!("{value}%")
        } else {
            value.to_string()
        }
    }
}

impl TryFrom<&str> for Percent {
    type Error = ();

    fn try_from(percent_str: &str) -> Result<Percent, ()> {
        let parse = |signed, value_str: &str| {
            match value_str.parse::<u8>() {
                Ok(value) => {
                    if value <= 100 {
                        Ok(Percent::new(signed, value))
                    } else {
                        Err(())
                    }
                }
                Err(_) => Err(())
            }
        };

        match percent_str.strip_suffix('%') {
            Some(leading_str) => parse(true, leading_str),
            None => parse(false, percent_str)
        }
    }
}

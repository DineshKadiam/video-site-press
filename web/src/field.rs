// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

pub struct Field {
    pub error: Option<String>,
    pub value: String
}

impl Field {
    pub fn valid(value: impl Into<String>) -> Field {
        Field {
            error: None,
            value: value.into()
        }
    }

    pub fn validated(
        value: impl Into<String>,
        error: Option<impl Into<String>>
    ) -> Field {
        Field {
            error: error.map(|message| message.into()),
            value: value.into()
        }
    }
}

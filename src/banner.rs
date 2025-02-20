// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{FileMeta, PosterAssets};

#[derive(Clone, Debug)]
pub struct Banner {
    pub assets: Option<PosterAssets>,
    pub description: Option<String>,
    pub file_meta: FileMeta,
    pub file_name: String
}

impl Banner {
    pub fn new(
        assets: Option<PosterAssets>,
        description: Option<String>,
        file_meta: FileMeta,
        file_name: String
    ) -> Banner {
        Banner {
            assets,
            description,
            file_meta,
            file_name
        }
    }
}
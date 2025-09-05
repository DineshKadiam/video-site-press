// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{FileMeta, PosterAssets, PosterMeta};

#[derive(Clone, Debug)]
pub struct PosterFile {
    pub assets: Option<PosterAssets>,
    pub description: Option<String>,
    pub file_meta: FileMeta,
    pub file_name: String,
    pub poster_meta: Option<PosterMeta>
}

impl PosterFile {
    pub fn new(
        assets: Option<PosterAssets>,
        description: Option<String>,
        file_meta: FileMeta,
        file_name: String,
        poster_meta: Option<PosterMeta>
    ) -> PosterFile {
        PosterFile {
            assets,
            description,
            file_meta,
            file_name,
            poster_meta
        }
    }
}

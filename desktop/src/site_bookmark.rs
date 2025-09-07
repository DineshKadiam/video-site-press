// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

pub struct SiteBookmark {
    pub path: PathBuf
}

impl SiteBookmark {
    pub fn new(path: impl Into<PathBuf>) -> SiteBookmark {
        // TODO: Add and implement "preview/probe" functionality, i.e.
        //       getting and storing at least the title and public address

        SiteBookmark {
            path: path.into()
        }
    }
}

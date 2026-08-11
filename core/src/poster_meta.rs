// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::hash::{DefaultHasher, Hash};
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use hyper8_util::hash_closure;

use crate::{
    Context,
    FileMeta,
    Lqip,
    PosterSourceCached,
    SitePath
};

/// Image-specific metadata asynchronously computed for the poster source image
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PosterMeta {
    pub height: u32,
    pub lqip: Lqip,
    pub width: u32
}

impl PosterMeta {
    /// Increment this cache serialization key when:
    /// - The data layout of [PosterMeta] changes
    /// - The way we compute [PosterMeta] changes
    ///
    /// This automatically lets the cache discard poster assets manifests that
    /// hold incorrect, incompatible or outdated data.
    pub const CACHE_SERIALIZATION_KEY: &'static str = "poster_source1";

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn deserialize_cached(path: &Path) -> Option<PosterSourceCached> {
        match fs::read(path) {
            Ok(bytes) => bincode::deserialize::<PosterSourceCached>(&bytes).ok(),
            Err(_) => None
        }
    }

    pub fn write_cache_manifest(
        &self,
        context: &Context,
        file_meta: &FileMeta,
        file_name: &str,
        site_path: &SitePath
    ) {
        // TODO: These "paths" are actually abstract paths, used to retrieve
        // the cached items under a certain "id" (which is this "path")
        // but the code does not make that clear. (but should!) Here and else-
        // where needs refactoring.
        let poster_path = format!("{}{}", site_path.normalized(), file_name);

        let cache_item = PosterSourceCached {
            file_meta: file_meta.clone(),
            path: poster_path.clone(),
            poster_meta: self.clone()
        };

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let hash = |hasher: &mut DefaultHasher| {
            poster_path.hash(hasher);
            file_meta.hash(hasher);
        };

        let cache_item_stem = hash_closure(hash);
        let cache_item_key = PosterMeta::CACHE_SERIALIZATION_KEY;

        let cache_item_file_name = format!("{cache_item_stem}.{cache_item_key}.bincode");
        let cache_item_path = context.cache_dir.join(cache_item_file_name);

        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}

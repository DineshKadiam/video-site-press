// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::hash::{DefaultHasher, Hash};

use serde_derive::{Deserialize, Serialize};

use crate::{
    CacheItem,
    Context,
    FileMeta,
    Lqip,
    PosterSourceCached,
    SitePath
};
use crate::util::hash_closure;

/// Image-specific metadata asynchronously computed for the poster source image
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PosterMeta {
    pub height: u32,
    pub lqip: Lqip,
    pub width: u32
}

impl PosterMeta {
    /// Increase version on each change to the data layout of [PosterMeta].
    /// This automatically informs the cache not to try to deserialize
    /// manifests that hold old, incompatible data.
    pub const CACHE_SERIALIZATION_KEY: &'static str = "poster_meta1";

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

        let cache_item = CacheItem::PosterSource(PosterSourceCached {
            file_meta: file_meta.clone(),
            path: poster_path.clone(),
            poster_meta: self.clone()
        });

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let hash = |hasher: &mut DefaultHasher| {
            poster_path.hash(hasher);
            file_meta.hash(hasher);
        };

        let cache_item_file_name = format!("{}.{}.bincode", PosterMeta::CACHE_SERIALIZATION_KEY, hash_closure(hash));
        let cache_item_path = context.cache_dir.join(cache_item_file_name);

        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}

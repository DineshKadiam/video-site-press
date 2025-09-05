// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::hash::{DefaultHasher, Hash};
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::{
    Context,
    FileMeta,
    PlatformIntegrationImage,
    PosterAssetsCached,
    SitePath
};
use crate::util::hash_closure;

pub struct ImgAttributes {
    pub src: String,
    pub srcset: String
}

/// A single, resized version of the poster image
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PosterAsset {
    pub file_name: String,
    pub filesize_bytes: u64,
    pub height: u32,
    pub width: u32
}

/// Represents multiple, differently sized versions of a poster image, for
/// display on different screen sizes
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PosterAssets {
    pub max_160: PosterAsset,
    pub max_320: Option<PosterAsset>,
    pub max_480: Option<PosterAsset>,
    pub max_800: Option<PosterAsset>,
    pub max_1280: Option<PosterAsset>,
    pub poster_aspect: f32
}

impl ImgAttributes {
    /// Assets MUST be passed in ascending size
    pub fn new(
        assets_ascending_by_size: Vec<&PosterAsset>,
        base_name: &str,
        prefix: &str
    ) -> ImgAttributes {
        let mut src = String::new();
        let mut srcset = Vec::new();

        let mut asset_peek_iter = assets_ascending_by_size.iter().peekable();

        while let Some(asset) = asset_peek_iter.next() {
            let target_filename = asset.target_filename(base_name);

            let width = asset.width;
            srcset.push(format!("{prefix}{target_filename} {width}w"));

            if asset_peek_iter.peek().is_none() {
                src = format!("{prefix}{target_filename}");
            }
        }

        ImgAttributes {
            src,
            srcset: srcset.join(",")
        }
    }
}

impl PosterAsset {
    /// base_name is either "banner" or "poster"
    pub fn target_filename(&self, base_name: &str) -> String {
        let height = self.height;
        let width = self.width;

        format!("{base_name}_{width}x{height}.jpg")
    }
}

impl PosterAssets {
    /// Increment this cache serialization key when:
    /// - The data layout of [PosterAssets] changes
    /// - The way we compute [PosterAssets] changes
    ///
    /// This automatically lets the cache discard poster assets manifests that
    /// hold incorrect, incompatible or outdated data.
    pub const CACHE_SERIALIZATION_KEY: &'static str = "poster_assets1";

    pub fn all(&self) -> Vec<&PosterAsset> {
        let mut result = Vec::with_capacity(4);

        result.push(&self.max_160);
        if let Some(asset) = &self.max_320 { result.push(asset); }
        if let Some(asset) = &self.max_480 { result.push(asset); }
        if let Some(asset) = &self.max_800 { result.push(asset); }
        if let Some(asset) = &self.max_1280 { result.push(asset); }

        result
    }

    pub fn deserialize_cached(path: &Path) -> Option<PosterAssetsCached> {
        match fs::read(path) {
            Ok(bytes) => bincode::deserialize::<PosterAssetsCached>(&bytes).ok(),
            Err(_) => None
        }
    }

    pub fn img_attributes_up_to_480(&self, base_name: &str, prefix: &str) -> ImgAttributes {
        let assets = match &self.max_320 {
            Some(max_320) => match &self.max_480 {
                Some(max_480) => vec![&self.max_160, max_320, max_480],
                None => vec![&self.max_160, max_320]
            }
            None => vec![&self.max_160]
        };

        ImgAttributes::new(assets, base_name, prefix)
    }

    pub fn img_attributes_up_to_1280(&self, base_name: &str, prefix: &str) -> ImgAttributes {
        let mut assets = Vec::with_capacity(4);

        assets.push(&self.max_160);
        if let Some(asset) = &self.max_320 { assets.push(asset); }
        if let Some(asset) = &self.max_480 { assets.push(asset); }
        if let Some(asset) = &self.max_800 { assets.push(asset); }
        if let Some(asset) = &self.max_1280 { assets.push(asset); }

        ImgAttributes::new(assets, base_name, prefix)
    }

    pub fn largest(&self) -> &PosterAsset {
        if let Some(max_1280) = &self.max_1280 {
            max_1280
        } else if let Some(max_800) = &self.max_800 {
            max_800
        } else if let Some(max_480) = &self.max_480 {
            max_480
        } else if let Some(max_320) = &self.max_320 {
            max_320
        } else {
            &self.max_160
        }
    }

    /// base_name is either "banner" or "poster"
    pub fn platform_integration_image(&self, base_name: &str, url_prefix: &String) -> PlatformIntegrationImage {
        let asset = if let Some(max_800) = &self.max_800 {
            max_800
        } else if let Some(max_480) = &self.max_480 {
            max_480
        } else if let Some(max_320) = &self.max_320 {
            max_320
        } else {
            &self.max_160
        };

        let target_filename = asset.target_filename(base_name);

        PlatformIntegrationImage {
            height: asset.height,
            url: format!("{url_prefix}{target_filename}"),
            width: asset.width
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
        // but the code does not make that clear. (but should!) Here and else
        // where needs refactoring.
        let poster_path = format!("{}{}", site_path.normalized(), file_name);

        let cache_item = PosterAssetsCached {
            assets: self.clone(),
            file_meta: file_meta.clone(),
            path: poster_path.clone()
        };

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let hash = |hasher: &mut DefaultHasher| {
            poster_path.hash(hasher);
            file_meta.hash(hasher);
            self.poster_aspect.to_string().hash(hasher);
        };

        let cache_item_stem = hash_closure(hash);
        let cache_item_key = PosterAssets::CACHE_SERIALIZATION_KEY;

        let cache_item_file_name = format!("{cache_item_stem}.{cache_item_key}.bincode");
        let cache_item_path = context.cache_dir.join(cache_item_file_name);

        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
}

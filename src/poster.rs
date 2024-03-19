use std::fs;

use serde_derive::{Deserialize, Serialize};

use crate::{CacheItem, Context, FileMeta, PosterAssetsCached};
use crate::util::hash;

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
    pub max_1280: Option<PosterAsset>
}

#[derive(Clone, Debug)]
pub struct PosterFile {
    pub assets: Option<PosterAssets>,
    pub file_meta: FileMeta,
    pub file_name: String
}

impl PosterAssets {
    pub fn write_manifest(&self, context: &Context, path: &str, poster: &PosterFile) {
        let poster_path = format!("{}/{}", path, poster.file_name);

        let cache_item = CacheItem::PosterAssets(PosterAssetsCached {
            assets: self.clone(),
            file_meta: poster.file_meta.clone(),
            path: poster_path.clone()
        });

        let cache_item_serialized = bincode::serialize(&cache_item).unwrap();

        let cache_item_file_name = format!("{}.bincode", hash(&poster_path));
        let cache_item_path = context.cache_dir.join(cache_item_file_name);
        
        fs::write(cache_item_path, cache_item_serialized).unwrap();
    }
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
            let height = asset.height;
            let width = asset.width;

            srcset.push(format!("{prefix}{base_name}_{width}x{height}.jpg {width}w"));

            if asset_peek_iter.peek().is_none() {
                src = format!("{prefix}{base_name}_{width}x{height}.jpg");
            }
        }

        ImgAttributes {
            src,
            srcset: srcset.join(",")
        }
    }
}

impl PosterAssets {
    pub fn all(&self) -> Vec<&PosterAsset> {
        let mut result = Vec::with_capacity(4);

        result.push(&self.max_160);
        if let Some(asset) = &self.max_320 { result.push(asset); }
        if let Some(asset) = &self.max_480 { result.push(asset); }
        if let Some(asset) = &self.max_800 { result.push(asset); }
        if let Some(asset) = &self.max_1280 { result.push(asset); }

        result
    }

    pub fn img_attributes_up_to_320(&self, base_name: &str, prefix: &str) -> ImgAttributes {
        let assets = match &self.max_320 {
            Some(max_320) => vec![&self.max_160, max_320],
            None => vec![&self.max_160]
        };

        ImgAttributes::new(assets, base_name, prefix)
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
}

impl PosterFile {
    pub fn new(
        assets: Option<PosterAssets>,
        file_meta: FileMeta,
        file_name: String
    ) -> PosterFile {
        PosterFile {
            assets,
            file_meta,
            file_name
        }
    }
}
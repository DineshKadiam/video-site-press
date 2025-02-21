// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Collection, Playlist, Video};

/// Some options need to be evaluated in the context of the entire site tree
/// before they are applied at the individual node level. This includes
/// embedding and download, where the option is inherited down the tree but
/// potentially overriden at the node level, as well as the unlisted option,
/// which unconditionally applies to all descendants of a site tree branch
/// whose(in)direct parent node is marked as unlisted.
///
/// MergeOptions represents these options, and at the node level we only
/// consider the options on this struct (after we have merged in the options
/// from the node level beforehand of course).
#[derive(Clone, Debug)]
pub struct MergedOptions {
    pub download: bool,
    pub embedding: bool,
    pub unlisted: bool
}

impl MergedOptions {
    /// Mutably merges this set of merged options with the collection's
    /// options by mutating this set itself.
    pub fn apply_collection_options(&mut self, collection: &Collection) {
        if let Some(download) = collection.download {
            self.download = download;
        }

        if let Some(embedding) = collection.embedding {
            self.embedding = embedding;
        }

        self.unlisted |= collection.unlisted;
    }

    /// Mutably merges this set of merged options with the playlist's
    /// options by mutating this set itself.
    pub fn apply_playlist_options(&mut self, playlist: &Playlist) {
        if let Some(download) = playlist.download {
            self.download = download;
        }

        if let Some(embedding) = playlist.embedding {
            self.embedding = embedding;
        }

        self.unlisted |= playlist.unlisted;
    }

    /// Mutably merges this set of merged options with the video's
    /// options by mutating this set itself.
    pub fn apply_video_options(&mut self, video: &Video) {
        if let Some(download) = video.download {
            self.download = download;
        }

        if let Some(embedding) = video.embedding {
            self.embedding = embedding;
        }

        self.unlisted |= video.unlisted;
    }

    /// This reflects the general defaults for a site in Hyper 8.
    pub fn default() -> MergedOptions {
        MergedOptions {
            download: false,
            embedding: false,
            unlisted: false
        }
    }

    /// Immutably merges this set of merged options with the collection's
    /// options by returning a new owned set of merged options.
    pub fn merge_collection_options(&self, collection: &Collection) -> MergedOptions {
        let mut merged_options = self.clone();
        merged_options.apply_collection_options(collection);
        merged_options
    }

    /// Immutably merges this set of merged options with the playlist's
    /// options by returning a new owned set of merged options.
    pub fn merge_playlist_options(&self, playlist: &Playlist) -> MergedOptions {
        let mut merged_options = self.clone();
        merged_options.apply_playlist_options(playlist);
        merged_options
    }

    /// Immutably merges this set of merged options with the video's
    /// options by returning a new owned set of merged options.
    pub fn merge_video_options(&self, video: &Video) -> MergedOptions {
        let mut merged_options = self.clone();
        merged_options.apply_video_options(video);
        merged_options
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    Collection,
    PlatformIntegration,
    Playlist
};

/// Some options need to be evaluated in the context of the entire site tree
/// before they are applied at the individual node level. This includes
/// copy_link, embedding and download, where the option is inherited down the
/// tree but potentially overriden at the node level, as well as the offline
/// and unlisted options, which unconditionally apply to all descendants of a
/// site tree branch whose (in)direct parent nodes are marked as either of the
/// two.
#[derive(Clone, Debug)]
pub struct InheritedOptions {
    pub copy_link: bool,
    pub download: bool,
    pub embedding: bool,
    pub feeds: bool,
    pub offline: bool,
    pub platform_integration: PlatformIntegration,
    pub unlisted: bool
}

impl InheritedOptions {
    /// Mutably merges this set of inherited options with the collection's
    /// options by mutating this set itself.
    pub fn apply_collection_options(&mut self, collection: &Collection) {
        if let Some(copy_link) = collection.copy_link_local() {
            self.copy_link = copy_link;
        }

        if let Some(download) = collection.download_local() {
            self.download = download;
        }

        if let Some(embedding) = collection.embedding_local() {
            self.embedding = embedding;
        }

        if let Some(feeds) = collection.feeds_local() {
            self.feeds = feeds;
        }

        self.offline |= collection.offline_local();

        if let Some(platform_integration) = collection.platform_integration_local() {
            self.platform_integration = platform_integration;
        }

        self.unlisted |= collection.unlisted_local();
    }

    /// Mutably merges this set of inherited options with the playlist's
    /// options by mutating this set itself.
    pub fn apply_playlist_options(&mut self, playlist: &Playlist) {
        if let Some(copy_link) = playlist.copy_link_local() {
            self.copy_link = copy_link;
        }

        if let Some(download) = playlist.download_local() {
            self.download = download;
        }

        if let Some(embedding) = playlist.embedding_local() {
            self.embedding = embedding;
        }

        if let Some(feeds) = playlist.feeds_local() {
            self.feeds = feeds;
        }

        self.offline |= playlist.offline_local();

        if let Some(platform_integration) = playlist.platform_integration_local() {
            self.platform_integration = platform_integration;
        }

        self.unlisted |= playlist.unlisted_local();
    }

    /// This reflects the general defaults for a site in Hyper 8.
    pub fn default() -> InheritedOptions {
        InheritedOptions {
            copy_link: true,
            download: false,
            embedding: false,
            feeds: true,
            offline: false,
            platform_integration: PlatformIntegration::Disabled,
            unlisted: false
        }
    }

    /// Immutably merges this set of inherited options with the collection's
    /// options by returning a new owned set of inherited options.
    pub fn merge_collection_options(&self, collection: &Collection) -> InheritedOptions {
        let mut inherited_options = self.clone();
        inherited_options.apply_collection_options(collection);
        inherited_options
    }

    /// Immutably merges this set of inherited options with the playlist's
    /// options by returning a new owned set of inherited options.
    pub fn merge_playlist_options(&self, playlist: &Playlist) -> InheritedOptions {
        let mut inherited_options = self.clone();
        inherited_options.apply_playlist_options(playlist);
        inherited_options
    }
}

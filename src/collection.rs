// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::{self, Ordering};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use url::Url;

use crate::{
    Banner,
    CollectionUpdate,
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    InheritedOptions,
    Link,
    PlatformIntegration,
    Playlist,
    PlaylistUpdate,
    Site,
    SitePath,
    Translations,
    TreeViewOrder,
    Video,
    VideoOrder,
    VideoUpdate
};
use crate::context::VideoParent;
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::checked_remove_dir_all;

const COLLECTION_OPTIONS: &[&str] = &[
    "banner",
    "copy_link",
    "description",
    "download",
    "embedding",
    "feeds",
    "link",
    "offline",
    "platform_integration",
    "title",
    "unlisted",
    "video_order"
];

#[derive(Clone, Debug)]
pub struct Collection {
    pub banner: Option<Banner>,
    copy_link: Option<bool>,
    pub description: Option<String>,
    /// Errors related to files within the collection directory, outside of
    /// the collection manifest which uses `manifest_errors`.
    pub dir_errors: Vec<String>,
    download: Option<bool>,
    embedding: Option<bool>,
    feeds: Option<bool>,
    /// Some options of a collection (e.g. copy_link, embedding and unlisted)
    /// are affected by both the local settings in the manifest as well as
    /// the inherited state from ancestors in the site tree. This stores
    /// the options interited by this collection from its ancestors.
    inherited_options: InheritedOptions,
    pub links: Vec<Link>,
    /// Syntactical and semantic errors inside the manifest for displaying to
    /// the user as well as to prevent programmatic updates to the site and
    /// its manifest while such errors are present.
    pub manifest_errors: Vec<String>,
    offline: bool,
    platform_integration: Option<PlatformIntegration>,
    pub playlists: Vec<Arc<Playlist>>,
    pub site_path: SitePath,
    pub subcollections: Vec<Arc<Collection>>,
    pub title: Option<String>,
    unlisted: bool,
    pub video_order: VideoOrder,
    pub videos: Vec<Arc<Video>>
}

impl Collection {
    pub fn all_metadata_computed(&self) -> bool {
        (self.banner.is_none() || self.banner.as_ref().is_some_and(|banner| banner.assets.is_some())) &&
        self.published_playlists().all(|playlist| playlist.all_metadata_computed()) &&
        self.published_subcollections().all(|subcollection| subcollection.all_metadata_computed()) &&
        self.published_videos().all(|video| video.all_metadata_computed())
    }

    fn apply_manifest(&mut self, context: &Context, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.manifest_errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return
            }
        };

        for element in document.elements() {
            match element.key() {
                "banner" => 'banner: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(attributes) = field.attributes() {
                            let mut description = None;
                            let mut file = None;

                            for attribute in attributes {
                                match attribute.key() {
                                    "description" => {
                                        if let Some(value) = attribute.value() {
                                            description = Some(value.to_string());
                                        }
                                    }
                                    "file" => {
                                        if let Some(value) = attribute.value() {
                                            let absolute_path = manifest_path.parent().unwrap().join(value);

                                            if absolute_path.exists() {
                                                file = Some((absolute_path, value.to_string()));
                                            } else {
                                                let message = format!("The referenced file was not found ({})", absolute_path.display());
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                    _ => {
                                        let message = "The key/name of this attribute was not recognized, only 'description' and 'file' are recognized inside an banner field";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            if let Some((absolute_path, file_name)) = file {
                                let file_meta = FileMeta::new(&absolute_path);

                                let assets = context.cache.lock().unwrap().fetch_poster_assets(
                                    &file_meta,
                                    &file_name,
                                    0.0, // TODO: Banners have no poster_aspect, pending better modeling we just hard-code it to 0 always
                                    &self.site_path
                                );

                                let banner = Banner::new(assets, description, file_meta, file_name);
                                self.banner = Some(banner);
                            }

                            break 'banner;
                        }
                    }

                    let message = "banner needs to be provided as a field with attributes, e.g.:\n\nbanner:\ndescription = Strange geometric patterns\nfile = patterns.jpg";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "copy_link" => 'copy_link: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.copy_link = Some(true);
                                    }
                                    "disabled" => {
                                        self.copy_link = Some(false);
                                    }
                                    other => {
                                        let message = format!("The copy_link option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'copy_link;
                            }
                        }
                    }

                    let message = "copy_link needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'copy_link: disabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "description" => {
                    if let Ok(embed) = element.as_embed() {
                        if let Some(value) = embed.value() {
                            self.description = Some(value.to_string());
                        } else {
                            self.description = None;
                        }
                    } else {
                        let message = "The 'description' option needs to be provided as an embed, e.g.:\n-- description\nA description text\n--description";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "download" => 'download: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.download = Some(true);
                                    }
                                    "disabled" => {
                                        self.download = Some(false);
                                    }
                                    other => {
                                        let message = format!("The download option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'download;
                            }
                        }
                    }

                    let message = "download needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'download: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "embedding" => 'embedding: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.embedding = Some(true);
                                    }
                                    "disabled" => {
                                        self.embedding = Some(false);
                                    }
                                    other => {
                                        let message = format!("The embedding option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'embedding;
                            }
                        }
                    }

                    let message = "embedding needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'embedding: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "feeds" => 'feeds: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match value {
                                    "enabled" => {
                                        self.feeds = Some(true);
                                    }
                                    "disabled" => {
                                        self.feeds = Some(false);
                                    }
                                    other => {
                                        let message = format!("The feeds option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'feeds;
                            }
                        }
                    }

                    let message = "feeds needs to be provided as a field with the value 'enabled' or 'disabled', e.g.: 'feeds: enabled'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "link" => 'link: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match Url::parse(value) {
                                    Ok(_) => {
                                        let link = Link::new(false, None, false, value);
                                        self.links.push(link);
                                    }
                                    Err(err) => {
                                        let message = format!("The url supplied for the link seems to be malformed ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            break 'link;
                        } else if let Ok(attributes) = field.attributes() {
                            let mut hidden = false;
                            let mut label = None;
                            let mut rel_me = false;
                            let mut url = None;

                            for attribute in attributes {
                                match attribute.key() {
                                    "label" => {
                                        if let Some(value) = attribute.value() {
                                            label = Some(value.to_string());
                                        }
                                    }
                                    "url" => {
                                        if let Some(value) = attribute.value() {
                                            match Url::parse(value) {
                                                Ok(_) => url = Some(value),
                                                Err(err) => {
                                                    let message = format!("The url supplied for the link seems to be malformed ({err})");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.manifest_errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    "verification" => {
                                        if let Some(value) = attribute.value() {
                                            match value {
                                                "rel-me" => {
                                                    hidden = false;
                                                    rel_me = true;
                                                }
                                                "rel-me-hidden" => {
                                                    hidden = true;
                                                    rel_me = true;
                                                }
                                                _ => {
                                                    let message = format!("The verification attribute value '{value}' is not recognized, allowed are 'rel-me' and 'rel-me-hidden'");
                                                    let error = attribute_error(attribute, manifest_path, &message);
                                                    self.manifest_errors.push(error);
                                                }
                                            }
                                        }
                                    }
                                    other => {
                                        let message = format!("The attribute '{other}' is not recognized here (supported attributes are 'label', 'url' and 'verification'");
                                        let error = attribute_error(attribute, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            if let Some(url) = url {
                                let link = Link::new(hidden, label, rel_me, url);
                                self.links.push(link);
                            } else {
                                let message = "The link option must supply an url attribute at least, e.g.:\n\nlink:\nurl = https://example.com";
                                let error = element_error(element, manifest_path, message);
                                self.manifest_errors.push(error);
                            }

                            break 'link;
                        }
                    }

                    let message = "link must be provided as a basic field with a value (e.g. 'link: https://example.com') or in its extended form as a field with attributes, e.g.:\n\nlink:\nurl = https://example.com\nlabel = Example";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "offline" => {
                    if element.is_flag() {
                        self.offline = true;
                    } else {
                        let message = "offline needs to be provided as a flag, that is, exactly as 'offline' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "platform_integration" => 'platform_integration: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match PlatformIntegration::from_key(value) {
                                    Some(platform_integration) => {
                                        self.platform_integration = Some(platform_integration);
                                    }
                                    None => {
                                        let message = format!("The platform_integration option only supports the value 'link_previews', 'content_embeds' or 'disabled'. The given value '{value}' was not recognized.");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }

                                break 'platform_integration;
                            }
                        }
                    }

                    let message = "platform_integration needs to be provided as a field with the value 'link_previews', 'content_embeds' or 'disabled', e.g.: 'platform_integration: link_previews'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "title" => 'title: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                self.title = Some(value.to_string());
                            }

                            break 'title;
                        }
                    }

                    let message = "title needs to be provided as a field with a value, e.g.: 'title: The 90ies clip collection'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                "unlisted" => {
                    if element.is_flag() {
                        self.unlisted = true;
                    } else {
                        let message = "unlisted needs to be provided as a flag, that is, exactly as 'unlisted' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "video_order" => 'video_order: {
                    if let Ok(field) = element.as_field() {
                        if let Ok(result) = field.value() {
                            if let Some(value) = result {
                                match VideoOrder::from_key(value) {
                                    Ok(order) => {
                                        self.video_order = order;
                                    }
                                    Err(err) => {
                                        let message = format!("The supplied order was not recognized ({err})");
                                        let error = element_error(element, manifest_path, &message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                            }

                            break 'video_order;
                        }
                    }

                    let message = "video_order needs to be provided as a field with a value, e.g.: 'video_order: release_date_asc'";
                    let error = element_error(element, manifest_path, message);
                    self.manifest_errors.push(error);
                }
                other => {
                    let message = not_supported_error("collection.eno", other, &[COLLECTION_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.manifest_errors.push(error);
                }
            }
        }
    }

    pub fn cmp_by_latest_release_date_desc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        b.latest_release_date().cmp(&a.latest_release_date())
    }

    // Collections without title are sorted to the front
    pub fn cmp_by_slug_then_title_asc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => a_title.cmp(b_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => a.slug().cmp(&b.slug())
        }
    }

    // Collections without title are sorted to the front
    pub fn cmp_by_slug_then_title_desc(a: &Arc<Collection>, b: &Arc<Collection>) -> Ordering {
        match (&a.title, &b.title) {
            (Some(a_title), Some(b_title)) => b_title.cmp(a_title),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => b.slug().cmp(&a.slug())
        }
    }

    /// When we convert between types (collection, playlist, video) we pass
    /// over existing inherited options using this method.
    pub fn copy_inherited_options(&self) -> InheritedOptions {
        self.inherited_options.clone()
    }

    /// Get the copy_link setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn copy_link(&self) -> bool {
        self.copy_link.unwrap_or(self.inherited_options.copy_link)
    }

    /// Get the copy_link setting as inherited from ancestors.
    pub fn copy_link_inherited(&self) -> bool {
        self.inherited_options.copy_link
    }

    /// Get the copy_link setting as set locally on the collection.
    pub fn copy_link_local(&self) -> Option<bool> {
        self.copy_link
    }

    /// Set the copy_link setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn copy_link_local_set(&mut self, enabled_option: Option<bool>) -> bool {
        let must_propagate =
            enabled_option.unwrap_or(self.inherited_options.copy_link) !=
            self.copy_link.unwrap_or(self.inherited_options.copy_link);

        self.copy_link = enabled_option;

        must_propagate
    }

    /// Recursively traverses the collection to determine if any published
    /// video in it has copy_link enabled. This information is consecutively
    /// used to determine whether we need to write clipboard css/js assets to the
    /// build.
    pub fn copy_link_requested(&self) -> bool {
        self.published_videos().any(|video| video.copy_link()) ||
        self.published_playlists().any(|playlist| playlist.copy_link_requested()) ||
        self.published_subcollections().any(|subcollection| subcollection.copy_link_requested())
    }

    /// If the collection that is successfully deleted is a direct
    /// subcollection of this one, `Ok(None)` is returned, otherwise the
    /// deeper parent collection of the deleted collection is returned
    /// as `Ok(Some(deeper_parent))`.
    pub fn delete_collection(
        &mut self,
        context: &Context,
        site_path: &SitePath
    ) -> Result<Option<Arc<Collection>>, ()> {
        if let Some(subcollection) = self.subcollections
            .iter()
            .position(|subcollection| subcollection.site_path == *site_path)
            .map(|index| self.subcollections.swap_remove(index)) {
            subcollection.unpersist(context);

            return Ok(None);
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            if let Ok(deeper_parent) = subcollection_mut.delete_collection(context, site_path) {
                *subcollection = Arc::new(subcollection_mut);
                return Ok(deeper_parent.or(Some(Arc::clone(subcollection))));
            }
        }

        Err(())
    }

    /// If the playlist that is successfully deleted is directly
    /// contained in this collection, `Ok(None)` is returned, otherwise the
    /// deeper parent collection of the deleted playlist is returned
    /// as `Ok(Some(deeper_parent))`.
    pub fn delete_playlist(
        &mut self,
        context: &Context,
        site_path: &SitePath
    ) -> Result<Option<Arc<Collection>>, ()> {
        if let Some(playlist) = self.playlists
            .iter()
            .position(|playlist| playlist.site_path == *site_path)
            .map(|index| self.playlists.swap_remove(index)) {
            playlist.unpersist(context);

            return Ok(None);
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            if let Ok(deeper_parent) = subcollection_mut.delete_playlist(context, site_path) {
                *subcollection = Arc::new(subcollection_mut);
                return Ok(deeper_parent.or(Some(Arc::clone(subcollection))));
            }
        }

        Err(())
    }

    /// If the video that is successfully deleted is directly
    /// contained in this collection, `Ok(None)` is returned, otherwise the
    /// deeper parent of the deleted video is returned
    /// as `Ok(Some(deeper_parent))`.
    pub fn delete_video(
        &mut self,
        context: &Context,
        site_path: &SitePath
    ) -> Result<Option<VideoParent>, ()> {
        if let Some(video) = self.videos
            .iter()
            .position(|video| video.site_path == *site_path)
            .map(|index| self.videos.swap_remove(index)) {
            video.unpersist(context);

            return Ok(None);
        } else if let Some(playlist) = self.playlists
            .iter_mut()
            .find(|playlist| site_path.starts_with(&playlist.site_path)) {
            let mut playlist_mut = (**playlist).clone();

            if playlist_mut.delete_video(context, site_path) {
                *playlist = Arc::new(playlist_mut);
                return Ok(Some(VideoParent::Playlist(Arc::clone(playlist))));
            }
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            if let Ok(deeper_parent) = subcollection_mut.delete_video(context, site_path) {
                *subcollection = Arc::new(subcollection_mut);
                return Ok(deeper_parent.or(Some(VideoParent::Collection(Arc::clone(subcollection)))));
            }
        }

        Err(())
    }

    /// Get the download setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn download(&self) -> bool {
        self.download.unwrap_or(self.inherited_options.download)
    }

    /// Get the download setting as inherited from ancestors.
    pub fn download_inherited(&self) -> bool {
        self.inherited_options.download
    }

    /// Get the download setting as set locally on the collection.
    pub fn download_local(&self) -> Option<bool> {
        self.download
    }

    /// Set the download setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn download_local_set(&mut self, enabled_option: Option<bool>) -> bool {
        let must_propagate =
            enabled_option.unwrap_or(self.inherited_options.download) !=
            self.download.unwrap_or(self.inherited_options.download);

        self.download = enabled_option;

        must_propagate
    }

    /// Traverses the collection tree to determine if any published video in
    /// it needs to render embed codes. This information is used to check whether
    /// we need to write clipboard css/js assets to the build.
    pub fn embed_codes_requested(&self) -> bool {
        self.published_videos().any(|video| video.embedding()) ||
        self.published_playlists().any(|playlist| playlist.embed_codes_requested()) ||
        self.published_subcollections().any(|subcollection| subcollection.embed_codes_requested())
    }

    /// Get the embedding setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn embedding(&self) -> bool {
        self.embedding.unwrap_or(self.inherited_options.embedding)
    }

    /// Get the embedding setting as inherited from ancestors.
    pub fn embedding_inherited(&self) -> bool {
        self.inherited_options.embedding
    }

    /// Get the embedding setting as set locally on the collection.
    pub fn embedding_local(&self) -> Option<bool> {
        self.embedding
    }

    /// Set the embedding setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn embedding_local_set(&mut self, enabled_option: Option<bool>) -> bool {
        let must_propagate =
            enabled_option.unwrap_or(self.inherited_options.embedding) !=
            self.embedding.unwrap_or(self.inherited_options.embedding);

        self.embedding = enabled_option;

        must_propagate
    }

    /// Traverses the collection tree to determine if any published video in
    /// it needs to render embeds. This information is used to check whether
    /// we need to write embed css/js assets to the build.
    pub fn embeds_requested(&self) -> bool {
        self.published_videos().any(|video| video.embeds_requested()) ||
        self.published_playlists().any(|playlist| playlist.embeds_requested()) ||
        self.published_subcollections().any(|subcollection| subcollection.embeds_requested())
    }

    /// Get the feeds setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn feeds(&self) -> bool {
        self.feeds.unwrap_or(self.inherited_options.feeds)
    }

    /// Get the feeds setting as inherited from ancestors.
    pub fn feeds_inherited(&self) -> bool {
        self.inherited_options.feeds
    }

    /// Get the feeds setting as set locally on the collection.
    pub fn feeds_local(&self) -> Option<bool> {
        self.feeds
    }

    /// Set the feeds setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn feeds_local_set(&mut self, enabled_option: Option<bool>) -> bool {
        let must_propagate =
            enabled_option.unwrap_or(self.inherited_options.feeds) !=
            self.feeds.unwrap_or(self.inherited_options.feeds);

        self.feeds = enabled_option;

        must_propagate
    }

    /// Recursively traverses the collection to determine if any published
    /// video in it has feeds enabled. This information is consecutively
    /// used to determine whether we need to write clipboard css/js assets to the
    /// build.
    pub fn feeds_requested(&self) -> bool {
        self.published_playlists().any(|playlist| playlist.feeds()) ||
        self.published_subcollections().any(|subcollection| subcollection.feeds())
    }

    /// Returns a collection, playlist or video at the given site path.
    pub fn get_any(&self, site_path: &SitePath) -> GetAnyResult {
        for subcollection in &self.subcollections {
            if subcollection.site_path == *site_path {
                return GetAnyResult::Collection(Arc::clone(subcollection));
            } else if site_path.starts_with(&subcollection.site_path) {
                return subcollection.get_any(site_path);
            }
        }

        for playlist in &self.playlists {
            if playlist.site_path == *site_path {
                return GetAnyResult::Playlist(Arc::clone(playlist));
            } else if site_path.starts_with(&playlist.site_path) {
                return playlist.get_any(site_path);
            }
        }

        for video in &self.videos {
            if video.site_path == *site_path {
                return GetAnyResult::Video(Arc::clone(video));
            }
        }

        GetAnyResult::None
    }

    /// Returns the collection at the given site path.
    pub fn get_collection(&self, site_path: &SitePath) -> Option<Arc<Collection>> {
        for subcollection in &self.subcollections {
            if subcollection.site_path == *site_path {
                return Some(Arc::clone(subcollection));
            } else if site_path.starts_with(&subcollection.site_path) {
                return subcollection.get_collection(site_path);
            }
        }

        None
    }

    /// Returns the playlist at the given site path.
    pub fn get_playlist(&self, site_path: &SitePath) -> Option<Arc<Playlist>> {
        for playlist in &self.playlists {
            if playlist.site_path == *site_path {
                return Some(Arc::clone(playlist));
            }
        }

        for subcollection in &self.subcollections {
            if site_path.starts_with(&subcollection.site_path) {
                return subcollection.get_playlist(site_path);
            }
        }

        None
    }

    /// Returns the video at the given site path.
    pub fn get_video(&self, site_path: &SitePath) -> Option<Arc<Video>> {
        for video in &self.videos {
            if video.site_path == *site_path {
                return Some(Arc::clone(video));
            }
        }

        for playlist in &self.playlists {
            if site_path.starts_with(&playlist.site_path) {
                return playlist.get_video(site_path);
            }
        }

        for subcollection in &self.subcollections {
            if site_path.starts_with(&subcollection.site_path) {
                return subcollection.get_video(site_path);
            }
        }

        None
    }

    /// Sets new inherited options on the collection.
    pub fn inherited_options_set(&mut self, inherited_options: InheritedOptions) {
        self.inherited_options = inherited_options;
    }

    pub fn latest_release_date(&self) -> Option<NaiveDate> {
        let latest_in_playlists = self.public_playlists()
            .fold(None, |latest_release_date, playlist| {
                if let Some(release_date) = playlist.latest_release_date() {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::max(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        let latest_in_subcollections = self.public_subcollections()
            .fold(None, |latest_release_date, subcollection| {
                if let Some(release_date) = subcollection.latest_release_date() {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::max(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        let latest_in_videos = self.public_videos()
            .fold(None, |latest_release_date, video| {
                if let Some(release_date) = video.release_date {
                    if let Some(latest_release_date) = latest_release_date {
                        Some(cmp::max(latest_release_date, release_date))
                    } else {
                        Some(release_date)
                    }
                } else {
                    latest_release_date
                }
            });

        *[latest_in_playlists, latest_in_subcollections, latest_in_videos]
            .iter()
            .max()
            .unwrap()
    }

    /// Mutates the options the collection inherited from its ancestors by
    /// merging them with its own options, so they can be passed on to new
    /// descendants.
    pub fn mutate_inherited_options(&self) -> InheritedOptions {
        self.inherited_options.merge_collection_options(self)
    }

    pub fn new(inherited_options: InheritedOptions, site_path: SitePath) -> Collection {
        Collection {
            banner: None,
            copy_link: None,
            description: None,
            dir_errors: Vec::new(),
            download: None,
            embedding: None,
            feeds: None,
            inherited_options,
            links: Vec::new(),
            manifest_errors: Vec::new(),
            offline: false,
            platform_integration: None,
            playlists: Vec::new(),
            site_path,
            subcollections: Vec::new(),
            title: None,
            unlisted: false,
            video_order: VideoOrder::Default,
            videos: Vec::new()
        }
    }

    /// Get the offline setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn offline(&self) -> bool {
        self.offline || self.inherited_options.offline
    }

    /// Get the offline setting as inherited from ancestors.
    pub fn offline_inherited(&self) -> bool {
        self.inherited_options.offline
    }

    /// Get the offline setting as set locally on the collection.
    pub fn offline_local(&self) -> bool {
        self.offline
    }

    /// Set the offline setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn offline_local_set(&mut self, enabled: bool) -> bool {
        let must_propagate =
            (enabled || self.inherited_options.offline) !=
            (self.offline || self.inherited_options.offline);

        self.offline = enabled;

        must_propagate
    }

    // Calling this on a root node returns the same site path again
    pub fn parent_site_path_unchecked(&self) -> SitePath {
        self.site_path.parent_site_path_unchecked()
    }

    /// Get the platform_integration setting resolved in the context of the site tree
    /// (= as set locally on the colletion or as inherited from an ancestor).
    pub fn platform_integration(&self) -> PlatformIntegration {
        self.platform_integration.unwrap_or(self.inherited_options.platform_integration)
    }

    /// Get the platform_integration setting as inherited from ancestors.
    pub fn platform_integration_inherited(&self) -> PlatformIntegration {
        self.inherited_options.platform_integration
    }

    /// Get the platform_integration setting as set locally on the colletion.
    pub fn platform_integration_local(&self) -> Option<PlatformIntegration> {
        self.platform_integration
    }

    /// Set the platform_integration setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn platform_integration_local_set(&mut self, platform_integration_option: Option<PlatformIntegration>) -> bool {
        let must_propagate =
            platform_integration_option.unwrap_or(self.inherited_options.platform_integration) !=
            self.platform_integration.unwrap_or(self.inherited_options.platform_integration);

        self.platform_integration = platform_integration_option;

        must_propagate
    }

    /// Determines if the collection or any of its descendants has platform_integration
    /// enabled. This information is consecutively used to output a warning
    /// if base_url is not set (which is required for platform_integration meta to be
    /// generated).
    pub fn platform_integration_requested(&self) -> bool {
        self.platform_integration_local().is_some_and(|platform_integration| platform_integration.enabled()) ||
        self.published_videos().any(|video| video.platform_integration_local().is_some_and(|platform_integration| platform_integration.enabled())) ||
        self.published_playlists().any(|playlist| playlist.platform_integration_requested()) ||
        self.published_subcollections().any(|subcollection| subcollection.platform_integration_requested())
    }

    pub fn playlists_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Playlist>> {
        let mut playlists: Vec<Arc<Playlist>> = self.playlists.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => playlists.sort_unstable_by(Playlist::cmp_by_latest_release_date_desc),
            TreeViewOrder::TitleAsc => playlists.sort_unstable_by(Playlist::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => playlists.sort_unstable_by(Playlist::cmp_by_slug_then_title_desc)
        }

        playlists
    }

    /// Recursively clones all descendants of this collection "in place"
    /// (making them briefly mutable) to update their inherited setting
    /// states (download, embedding, offline, unlisted, etc.).
    pub fn propagate_options(&mut self) {
        let mutated_options = self.mutate_inherited_options();

        self.playlists = self.playlists
            .drain(..)
            .map(|playlist| {
                let mut playlist_mut = Arc::unwrap_or_clone(playlist);
                playlist_mut.inherited_options_set(mutated_options.clone());
                playlist_mut.propagate_options();
                Arc::new(playlist_mut)
            })
            .collect();

        self.subcollections = self.subcollections
            .drain(..)
            .map(|subcollection| {
                let mut subcollection_mut = Arc::unwrap_or_clone(subcollection);
                subcollection_mut.inherited_options_set(mutated_options.clone());
                subcollection_mut.propagate_options();
                Arc::new(subcollection_mut)
            })
            .collect();

        self.videos = self.videos
            .drain(..)
            .map(|video| {
                let mut video_mut = Arc::unwrap_or_clone(video);
                video_mut.inherited_options_set(mutated_options.clone());
                Arc::new(video_mut)
            })
            .collect();
    }

    /// A collection is public in its local context if it is neither offline
    /// nor unlisted according to its own settings. This does
    /// (intentionally) not take into account publicity settings inherited
    /// from ancestors.
    pub fn public_local(&self) -> bool {
        !self.offline && !self.unlisted
    }

    pub fn public_playlists(&self) -> impl Iterator<Item = &Arc<Playlist>> {
        self.playlists.iter().filter(|playlist| playlist.public_local())
    }

    pub fn public_playlists_desc_by_release_date(&self) -> Vec<Arc<Playlist>> {
        let mut playlists: Vec<Arc<Playlist>> = self.public_playlists().cloned().collect();
        playlists.sort_unstable_by(Playlist::cmp_by_latest_release_date_desc);
        playlists
    }

    pub fn public_subcollections(&self) -> impl Iterator<Item = &Arc<Collection>> {
        self.subcollections.iter().filter(|subcollection| subcollection.public_local())
    }

    pub fn public_subcollections_desc_by_release_date(&self) -> Vec<Arc<Collection>> {
        let mut subcollections: Vec<Arc<Collection>> = self.public_subcollections().cloned().collect();
        subcollections.sort_unstable_by(Collection::cmp_by_latest_release_date_desc);
        subcollections
    }

    /// Returns all public videos in the collection, i.e. those that are not
    /// marked as offline or unlisted.
    pub fn public_videos(&self) -> impl Iterator<Item = &Arc<Video>> {
        self.videos.iter().filter(|video| video.public_local())
    }

    pub fn public_videos_desc_by_release_date(&self) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.public_videos().cloned().collect();
        videos.sort_unstable_by(Video::cmp_by_release_date_desc);
        videos
    }

    pub fn public_videos_in_collection_video_order(&self) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.public_videos().cloned().collect();
        videos.sort_unstable_by(Video::cmp_by_video_order(&self.video_order));
        videos
    }

    /// Returns all published playlists in the collection, i.e. those that are
    /// not marked as offline.
    pub fn published_playlists(&self) -> impl Iterator<Item = &Arc<Playlist>> {
        self.playlists.iter().filter(|playlist| !playlist.offline_local())
    }

    /// Returns all published subcollections in the collection, i.e. those
    /// that are not marked as offline.
    pub fn published_subcollections(&self) -> impl Iterator<Item = &Arc<Collection>> {
        self.subcollections.iter().filter(|subcollection| !subcollection.offline_local())
    }

    /// Returns all published videos in the collection, i.e. those that are
    /// not marked as offline.
    pub fn published_videos(&self) -> impl Iterator<Item = &Arc<Video>> {
        self.videos.iter().filter(|video| !video.offline_local())
    }

    /// Returns all public videos (i.e. those that are not marked as offline
    /// or unlisted) in the collection and its playlists and subcollections,
    /// recursively.
    pub fn public_videos_recursive(&self) -> Vec<Arc<Video>> {
        let mut videos = Vec::new();

        fn recursive_append(collection: &Collection, videos: &mut Vec<Arc<Video>>) {
            for subcollection in collection.public_subcollections() {
                recursive_append(subcollection, videos);
            }

            for playlist in collection.public_playlists() {
                for video in playlist.public_videos() {
                    videos.push(video.clone());
                }
            }

            for video in collection.public_videos() {
                videos.push(video.clone());
            }
        }

        recursive_append(self, &mut videos);

        // TODO: Sorting

        videos
    }

    pub fn read_dir(
        context: &Context,
        hyper_dir: &HyperDir,
        inherited_options: InheritedOptions,
        site: &Site
    ) -> Arc<Collection> {
        let site_path = hyper_dir.site_path(context);
        let mut collection = Collection::new(inherited_options, site_path);

        if let Some(manifest) = &hyper_dir.collection_manifest {
            collection.apply_manifest(context, manifest);
        }

        for subdir in &hyper_dir.dirs {
            let hyper_subdir = HyperDir::read(subdir);
            collection.read_subdir(context, &hyper_subdir, site);
        }

        // TODO: Some strategy what we do with multiple images (e.g. heuristic* to pick most probable banner, use additional ones, etc.)
        //       (*) largest image? image that best fits the aspect ratio? name convention? etc.
        if !hyper_dir.image_files.is_empty() && collection.banner.is_none() {
            let first_image = hyper_dir.image_files.first().unwrap();

            let file_meta = FileMeta::new(first_image);
            let file_name = first_image.file_name().unwrap().to_string_lossy().to_string();

            let assets = context.cache.lock().unwrap().fetch_poster_assets(
                &file_meta,
                &file_name,
                0.0,
                &collection.site_path
            );

            let banner = Banner::new(assets, None, file_meta, file_name);

            collection.banner = Some(banner);
        }

        if !hyper_dir.video_files.is_empty() {
            let mut error = "No video files should be directly in a collection directory, maybe you wanted to put them in a subfolder instead?".to_string();

            for video_file in &hyper_dir.video_files {
                error.push_str(&format!("\n- {}", video_file.display()));
            }

            collection.dir_errors.push(error);
        }

        if !hyper_dir.errors.is_empty() {
            let mut error = "Directories and/or files inside this collection directory have issues:".to_string();

            for message in &hyper_dir.errors {
                error.push_str(&format!("\n- {}", message));
            }

            collection.dir_errors.push(error);
        }

        Arc::new(collection)
    }

    fn read_subdir(&mut self, context: &Context, hyper_subdir: &HyperDir, site: &Site) {
        if hyper_subdir.collection_manifest.is_some() {
            if hyper_subdir.playlist_manifest.is_some() || hyper_subdir.video_manifest.is_some() {
                let error = format!("Ignoring collection subdirectory {} because it contains conflicting manifest types (only one of collection.eno, playlist.eno and video.eno may exist in one directory)", hyper_subdir.path.display());
                self.dir_errors.push(error);
            } else {
                let subcollection = Collection::read_dir(context, hyper_subdir, self.mutate_inherited_options(), site);
                self.subcollections.push(subcollection);
            }
        } else if hyper_subdir.playlist_manifest.is_some() {
            if hyper_subdir.video_manifest.is_some() {
                let error = format!("Ignoring collection subdirectory {} because it contains both playlist.eno and video.eno (only one of the two may exist in one directory)", hyper_subdir.path.display());
                self.dir_errors.push(error);
            } else {
                let playlist = Playlist::read_dir(context, hyper_subdir, self.mutate_inherited_options(), site);
                self.playlists.push(playlist);
            }
        } else if hyper_subdir.video_manifest.is_some() {
            let video = Video::read_dir(context, hyper_subdir, self.mutate_inherited_options(), site);
            self.videos.push(video);
        } else if !hyper_subdir.dirs.is_empty() {
            let subcollection = Collection::read_dir(context, hyper_subdir, self.mutate_inherited_options(), site);
            self.subcollections.push(subcollection);
        } else if !hyper_subdir.video_files.is_empty() {
            let video = Video::read_dir(context, hyper_subdir, self.mutate_inherited_options(), site);
            self.videos.push(video);
        } else {
            // TODO: We currently ignore an empty subdir, as if it didn't exist.
            //       Should we rather register it somehow (e.g. implicity as collection),
            //       or handle it as an "Empty" node as we do in SiteContent::Empty?, in
            //       order to communicate the existence via the brower interface too.
        }
    }

    pub fn slug(&self) -> Option<&str> {
        self.site_path.last_component()
    }

    pub fn slug_unchecked(&self) -> &str {
        self.site_path.last_component_unchecked()
    }

    pub fn subcollections_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Collection>> {
        let mut subcollections: Vec<Arc<Collection>> = self.subcollections.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => subcollections.sort_unstable_by(Collection::cmp_by_latest_release_date_desc),
            TreeViewOrder::TitleAsc => subcollections.sort_unstable_by(Collection::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => subcollections.sort_unstable_by(Collection::cmp_by_slug_then_title_desc)
        }

        subcollections
    }

    /// If the subscribe page permalink we have in our translations collides
    /// with any of the permalinks in this collection, we prepend underscores
    /// to it until there is no collision anymore.
    pub fn subscribe_permalink(&self, translations: &Translations) -> String {
        let mut subscribe_slug = translations.subscribe_permalink.to_string();

        loop {
            if !self.published_subcollections()
                    .any(|subcollection| subcollection.slug_unchecked() == subscribe_slug) &&
                !self.published_playlists()
                    .any(|playlist| playlist.slug_unchecked() == subscribe_slug) &&
                !self.published_videos()
                    .any(|video| video.slug_unchecked() == subscribe_slug) {
                return subscribe_slug;
            }

            subscribe_slug = format!("_{subscribe_slug}");
        }
    }

    pub fn title_or_slug_or_generic_label(&self, translations: &Translations) -> &str {
        match &self.title {
            Some(title) => title,
            None => self.slug().unwrap_or(translations.collection)
        }
    }

    /// Get the unlisted setting resolved in the context of the site tree
    /// (= as set locally on the collection or as inherited from an ancestor).
    pub fn unlisted(&self) -> bool {
        self.unlisted || self.inherited_options.unlisted
    }

    /// Get the unlisted setting as inherited from ancestors.
    pub fn unlisted_inherited(&self) -> bool {
        self.inherited_options.unlisted
    }

    /// Get the unlisted setting as set locally on the collection.
    pub fn unlisted_local(&self) -> bool {
        self.unlisted
    }

    /// Set the unlisted setting locally on the collection. Returns whether
    /// this change must be propagated to descendants due to a resulting
    /// change in inherited settings.
    pub fn unlisted_local_set(&mut self, enabled: bool) -> bool {
        let must_propagate =
            (enabled || self.inherited_options.unlisted) !=
            (self.unlisted || self.inherited_options.unlisted);

        self.unlisted = enabled;

        must_propagate
    }

    pub fn unpersist(&self, context: &Context) {
        if self.site_path.is_root_path() {
            let manifest_path = context.site_dir.join("collection.eno");
            let _ = fs::remove_file(&manifest_path);

            if let Some(banner) = &self.banner {
                let banner_path = context.site_dir.join(&banner.file_name);
                let _ = fs::remove_file(&banner_path);
            }

            for playlist in &self.playlists {
                let playlist_path = context.site_dir.join(playlist.slug_unchecked());
                let _ = checked_remove_dir_all(context, &playlist_path);
            }

            for subcollection in &self.subcollections {
                let subcollection_path = context.site_dir.join(subcollection.slug_unchecked());
                let _ = checked_remove_dir_all(context, &subcollection_path);
            }

            for video in &self.videos {
                let video_path = context.site_dir.join(video.slug_unchecked());
                let _ = checked_remove_dir_all(context, &video_path);
            }
        } else {
            let collection_dir = context.site_dir.join(self.site_path.filesystem_relative());
            let _ = checked_remove_dir_all(context, &collection_dir);
        }
    }

    /// The site path to be updated is intended to be that of a descendant of
    /// the collection, never the collection itself!
    pub fn update_descendant_from_filesystem(
        &mut self,
        context: &Context,
        site: &Site,
        site_path: &SitePath
    ) {
        if site_path.is_direct_descendant_of(&self.site_path) {
            // If we presently have a video, playlist or collection registered
            // at the given site path, we remove it.
            if let Some(index) = self.videos
                .iter()
                .position(|video| video.site_path == *site_path) {
                self.videos.swap_remove(index);
            } else if let Some(index) = self.playlists
                .iter()
                .position(|playlist| playlist.site_path == *site_path) {
                self.playlists.swap_remove(index);
            } else if let Some(index) = self.subcollections
                .iter()
                .position(|subcollection| subcollection.site_path == *site_path) {
                self.subcollections.swap_remove(index);
            }

            // Add the new leaf (respectively, replace what was removed)
            let subdir = context.site_dir.join(site_path.filesystem_relative());
            let hyper_subdir = HyperDir::read(&subdir);
            self.read_subdir(context, &hyper_subdir, site);
        } else {
            // Here we know the update occured "deeper down", so we
            // (mostly) just clone the tree nodes at this level and
            // delegate actual reads/updates to the lower leaves of the tree.

            if let Some(index) = self.videos
                .iter()
                .position(|video| site_path.starts_with(&video.site_path)) {
                // A video isn't allowed to have subfolders, so this is a bit of an odd case,
                // so we just remove and replace it with whatever happened down this branch.
                self.videos.swap_remove(index);
                let closest_descendant = self.site_path.closest_descendant_to_unchecked(site_path);
                let hyper_subdir = HyperDir::read(&context.site_dir.join(closest_descendant.filesystem_relative()));
                self.read_subdir(context, &hyper_subdir, site);
            } else if let Some(playlist) = self.playlists
                .iter_mut()
                .find(|playlist| site_path.starts_with(&playlist.site_path)) {
                let mut playlist_mut = (**playlist).clone();
                playlist_mut.update_descendant_from_filesystem(context, site, site_path);
                *playlist = Arc::new(playlist_mut);
            } else if let Some(subcollection) = self.subcollections
                .iter_mut()
                .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
                let mut subcollection_mut = (**subcollection).clone();
                subcollection_mut.update_descendant_from_filesystem(context, site, site_path);
                *subcollection = Arc::new(subcollection_mut);
            } else {
                // Here a sub-subdir needs to be updated (e.g. a video in a
                // collection within this collection) that we haven't
                // even yet registered as subdir, so we read the entire
                // subtree.

                let closest_descendant = self.site_path.closest_descendant_to_unchecked(site_path);
                let hyper_subdir = HyperDir::read(&context.site_dir.join(closest_descendant.filesystem_relative()));
                self.read_subdir(context, &hyper_subdir, site);
            }
        }
    }

    pub fn update_playlist<T>(
        &mut self,
        mutation: impl FnOnce(&mut Playlist) -> T,
        site_path: &SitePath
    ) -> PlaylistUpdate<T> {
        if let Some(playlist) = self.playlists
            .iter_mut()
            .find(|playlist| playlist.site_path == *site_path) {
            if playlist.manifest_errors.is_empty() {
                let mut playlist_mut = (**playlist).clone();
                let optional_return_value = mutation(&mut playlist_mut);
                *playlist = Arc::new(playlist_mut);

                PlaylistUpdate::Updated {
                    optional_return_value,
                    playlist_updated: Arc::clone(playlist)
                }
            } else {
                PlaylistUpdate::PendingErrors {
                    playlist: Arc::clone(playlist)
                }
            }
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            let deeper_result = subcollection_mut.update_playlist(mutation, site_path);

            if let PlaylistUpdate::Updated { .. } = deeper_result {
                *subcollection = Arc::new(subcollection_mut);
            }

            deeper_result
        } else {
            PlaylistUpdate::NotFound
        }
    }

    pub fn update_subcollection<T>(
        &mut self,
        mutation: impl FnOnce(&mut Collection) -> T,
        site_path: &SitePath
    ) -> CollectionUpdate<T> {
        if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| subcollection.site_path == *site_path) {
            if subcollection.manifest_errors.is_empty() {
                let mut subcollection_mut = (**subcollection).clone();
                let optional_return_value = mutation(&mut subcollection_mut);
                *subcollection = Arc::new(subcollection_mut);

                CollectionUpdate::Updated {
                    collection_updated: Arc::clone(subcollection),
                    optional_return_value
                }
            } else {
                CollectionUpdate::PendingErrors {
                    collection: Arc::clone(subcollection)
                }
            }
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            let deeper_result = subcollection_mut.update_subcollection(mutation, site_path);

            if let CollectionUpdate::Updated { .. } = deeper_result {
                *subcollection = Arc::new(subcollection_mut);
            }

            deeper_result
        } else {
            CollectionUpdate::NotFound
        }
    }

    pub fn update_video<T>(
        &mut self,
        mutation: impl FnOnce(&mut Video) -> Result<T, String>,
        site_path: &SitePath
    ) -> VideoUpdate<T> {
        if let Some(video) = self.videos
            .iter_mut()
            .find(|video| video.site_path == *site_path) {
            if video.manifest_errors.is_empty() {
                let mut video_mut = (**video).clone();

                match mutation(&mut video_mut) {
                    Ok(optional_return_value) => {
                        *video = Arc::new(video_mut);

                        VideoUpdate::Success {
                            optional_return_value,
                            video_updated: Arc::clone(video)
                        }
                    }
                    Err(err) => VideoUpdate::Error(err)
                }
            } else {
                VideoUpdate::PendingErrors {
                    video: Arc::clone(video)
                }
            }
        } else if let Some(playlist) = self.playlists
            .iter_mut()
            .find(|playlist| site_path.starts_with(&playlist.site_path)) {
            let mut playlist_mut = (**playlist).clone();

            let deeper_result = playlist_mut.update_video(mutation, site_path);

            if let VideoUpdate::Success { .. } = deeper_result {
                *playlist = Arc::new(playlist_mut);
            }

            deeper_result
        } else if let Some(subcollection) = self.subcollections
            .iter_mut()
            .find(|subcollection| site_path.starts_with(&subcollection.site_path)) {
            let mut subcollection_mut = (**subcollection).clone();

            let deeper_result = subcollection_mut.update_video(mutation, site_path);

            if let VideoUpdate::Success { .. } = deeper_result {
                *subcollection = Arc::new(subcollection_mut);
            }

            deeper_result
        } else {
            VideoUpdate::NotFound
        }
    }

    pub fn videos_in_tree_view_order(&self, tree_view_order: &TreeViewOrder) -> Vec<Arc<Video>> {
        let mut videos: Vec<Arc<Video>> = self.videos.clone();

        match tree_view_order {
            TreeViewOrder::Default |
            TreeViewOrder::SiteOrder => videos.sort_unstable_by(Video::cmp_by_video_order(&self.video_order)),
            TreeViewOrder::TitleAsc => videos.sort_unstable_by(Video::cmp_by_slug_then_title_asc),
            TreeViewOrder::TitleDesc => videos.sort_unstable_by(Video::cmp_by_slug_then_title_desc)
        }

        videos
    }

    pub fn write_manifest(&self, context: &Context) {
        let mut eno = String::new();

        if let Some(banner) = &self.banner {
            eno.push_str("banner:\n");

            if let Some(description) = &banner.description {
                eno.push_str(&format!("description = {}\n", description));
            }

            eno.push_str(&format!("file = {}\n", banner.file_name));
        }

        if let Some(copy_link) = self.copy_link {
            eno.push_str(&format!("copy_link: {}\n", if copy_link { "enabled" } else { "disabled" }));
        }

        if let Some(download) = self.download {
            eno.push_str(&format!("download: {}\n", if download { "enabled" } else { "disabled" }));
        }

        if let Some(embedding) = self.embedding {
            eno.push_str(&format!("embedding: {}\n", if embedding { "enabled" } else { "disabled" }));
        }

        if self.offline {
            eno.push_str("offline\n");
        }

        if let Some(platform_integration) = self.platform_integration {
            eno.push_str(&format!("platform_integration: {}\n", platform_integration.key()));
        }

        if let Some(title) = &self.title {
            eno.push_str(&format!("title: {}\n", title));
        }

        if self.unlisted {
            eno.push_str("unlisted\n");
        }

        if self.video_order != VideoOrder::Default {
            eno.push_str(&format!("video_order: {}\n", self.video_order.key()));
        }

        for link in &self.links {
            eno.push_str(&link.to_eno());
        }

        if let Some(description) = &self.description {
            if !eno.is_empty() { eno.push('\n'); }
            eno.push_str(&format!("-- description\n{}\n-- description\n", description));
        }

        let path = context.site_dir
            .join(self.site_path.filesystem_relative())
            .join("collection.eno");

        fs::write(path, eno).unwrap();
    }
}

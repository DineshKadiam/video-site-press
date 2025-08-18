// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Routes in this file must be kept up to date with the mapping we keep in
//! `server.rs`!

use crate::{
    Collection,
    Playlist,
    SitePath,
    Video
};

pub fn collection_add_link(collection: &Collection) -> String {
    format!("/collection/add-link/{}", collection.site_path.normalized())
}

pub fn collection_banner(collection: &Collection) -> String {
    format!("/collection/banner/{}", collection.site_path.normalized())
}

pub fn collection_create_playlist(collection: &Collection) -> String {
    format!("/collection/create-playlist/{}", collection.site_path.normalized())
}

pub fn collection_create_subcollection(collection: &Collection) -> String {
    format!("/collection/create-subcollection/{}", collection.site_path.normalized())
}

pub fn collection_create_video(collection: &Collection) -> String {
    format!("/collection/create-video/{}", collection.site_path.normalized())
}

pub fn collection_dangerous_actions(collection: &Collection) -> String {
    format!("/collection/dangerous-actions/{}", collection.site_path.normalized())
}

pub fn collection_delete(collection: &Collection) -> String {
    format!("/collection/delete/{}", collection.site_path.normalized())
}

pub fn collection_delete_banner(collection: &Collection) -> String {
    format!("/collection/delete-banner/{}", collection.site_path.normalized())
}

pub fn collection_delete_link(collection: &Collection) -> String {
    format!("/collection/delete-link/{}", collection.site_path.normalized())
}

pub fn collection_edit(collection: &Collection) -> String {
    format!("/collection/edit/{}", collection.site_path.normalized())
}

pub fn collection_links(collection: &Collection) -> String {
    format!("/collection/links/{}", collection.site_path.normalized())
}

pub fn collection_manifest(collection: &Collection) -> String {
    format!("/collection/manifest/{}", collection.site_path.normalized())
}

pub fn collection_move(collection: &Collection) -> String {
    format!("/collection/move/{}", collection.site_path.normalized())
}

pub fn collection_update_banner(collection: &Collection) -> String {
    format!("/collection/update-banner/{}", collection.site_path.normalized())
}

pub fn collection_upload_banner(collection: &Collection) -> String {
    format!("/collection/upload-banner/{}", collection.site_path.normalized())
}

pub fn convert_to_collection(site_path: &SitePath) -> String {
    format!("/convert-to-collection/{}", site_path.normalized())
}

pub fn convert_to_playlist(site_path: &SitePath) -> String {
    format!("/convert-to-playlist/{}", site_path.normalized())
}

pub fn convert_to_video(site_path: &SitePath) -> String {
    format!("/convert-to-video/{}", site_path.normalized())
}

pub const DASHBOARD: &str = "/";

pub const DEPLOY: &str = "/deploy";

pub fn playlist_add_link(playlist: &Playlist) -> String {
    format!("/playlist/add-link/{}", playlist.site_path.normalized())
}

pub fn playlist_banner(playlist: &Playlist) -> String {
    format!("/playlist/banner/{}", playlist.site_path.normalized())
}

pub fn playlist_create_video(playlist: &Playlist) -> String {
    format!("/playlist/create-video/{}", playlist.site_path.normalized())
}

pub fn playlist_dangerous_actions(playlist: &Playlist) -> String {
    format!("/playlist/dangerous-actions/{}", playlist.site_path.normalized())
}

pub fn playlist_delete(playlist: &Playlist) -> String {
    format!("/playlist/delete/{}", playlist.site_path.normalized())
}

pub fn playlist_delete_banner(playlist: &Playlist) -> String {
    format!("/playlist/delete-banner/{}", playlist.site_path.normalized())
}

pub fn playlist_delete_link(playlist: &Playlist) -> String {
    format!("/playlist/delete-link/{}", playlist.site_path.normalized())
}

pub fn playlist_edit(playlist: &Playlist) -> String {
    format!("/playlist/edit/{}", playlist.site_path.normalized())
}

pub fn playlist_links(playlist: &Playlist) -> String {
    format!("/playlist/links/{}", playlist.site_path.normalized())
}

pub fn playlist_manifest(playlist: &Playlist) -> String {
    format!("/playlist/manifest/{}", playlist.site_path.normalized())
}

pub fn playlist_move(playlist: &Playlist) -> String {
    format!("/playlist/move/{}", playlist.site_path.normalized())
}

pub fn playlist_update_banner(playlist: &Playlist) -> String {
    format!("/playlist/update-banner/{}", playlist.site_path.normalized())
}

pub fn playlist_upload_banner(playlist: &Playlist) -> String {
    format!("/playlist/upload-banner/{}", playlist.site_path.normalized())
}

// TODO: This is nested in the video endpoint and says "poster", but currently
// this is called from a collection ("banner"), playlist ("banner") and video
// context, so semantically it would make sense to look at this again and
// either "monomorphize" it into three endpoints or extract the single
// universal endpoint from the video context.
pub fn poster(file_name: &str, site_path: &SitePath) -> String {
    format!("/poster/{}{}", site_path.normalized(),  urlencoding::encode(file_name))
}

pub const SITE_EDIT: &str = "/site/edit/";

pub const SITE_MANIFEST: &str = "/site/manifest/";

// TODO: This is nested in the video endpoint, but currently this is called
// from a collection, playlist and video context, so semantically it would
// make sense to look at this again and either "monomorphize" it into three
// endpoints or extract the single universal endpoint from the video context.
pub fn upload_video(site_path: &SitePath) -> String {
    format!("/video/upload-video/{}", site_path.normalized())
}

pub fn video_add_link(video: &Video) -> String {
    format!("/video/add-link/{}", video.site_path.normalized())
}

pub fn video_dangerous_actions(video: &Video) -> String {
    format!("/video/dangerous-actions/{}", video.site_path.normalized())
}

pub fn video_delete(video: &Video) -> String {
    format!("/video/delete/{}", video.site_path.normalized())
}

pub fn video_delete_audio_file(video: &Video) -> String {
    format!("/video/delete-audio-file/{}", video.site_path.normalized())
}

pub fn video_delete_link(video: &Video) -> String {
    format!("/video/delete-link/{}", video.site_path.normalized())
}

pub fn video_delete_poster(video: &Video) -> String {
    format!("/video/delete-poster/{}", video.site_path.normalized())
}

pub fn video_delete_subtitles(video: &Video) -> String {
    format!("/video/delete-subtitles/{}", video.site_path.normalized())
}

pub fn video_delete_version(video: &Video) -> String {
    format!("/video/delete-version/{}", video.site_path.normalized())
}

pub fn video_downsize(video: &Video) -> String {
    format!("/video/downsize/{}", video.site_path.normalized())
}

pub fn video_edit(video: &Video) -> String {
    format!("/video/edit/{}", video.site_path.normalized())
}

pub fn video_edit_subtitles(file_name: &str, video: &Video) -> String {
    format!("/video/edit-subtitles/{}/{}", urlencoding::encode(file_name), video.site_path.normalized())
}

pub fn video_links(video: &Video) -> String {
    format!("/video/links/{}", video.site_path.normalized())
}

pub fn video_manifest(video: &Video) -> String {
    format!("/video/manifest/{}", video.site_path.normalized())
}

pub fn video_media(video: &Video) -> String {
    format!("/video/media/{}", video.site_path.normalized())
}

pub fn video_move(video: &Video) -> String {
    format!("/video/move/{}", video.site_path.normalized())
}

pub fn video_pick_poster(video: &Video) -> String {
    format!("/video/pick-poster/{}", video.site_path.normalized())
}

pub fn video_poster(video: &Video) -> String {
    format!("/video/poster/{}", video.site_path.normalized())
}

pub fn video_stream(file_name: &str, video: &Video) -> String {
    format!("/video/stream/{}{}", video.site_path.normalized(), urlencoding::encode(file_name))
}

pub fn video_subtitles(video: &Video) -> String {
    format!("/video/subtitles/{}", video.site_path.normalized())
}

pub fn video_update_poster(video: &Video) -> String {
    format!("/video/update-poster/{}", video.site_path.normalized())
}

pub fn video_upload_poster(video: &Video) -> String {
    format!("/video/upload-poster/{}", video.site_path.normalized())
}

pub fn video_upload_subtitles(video: &Video) -> String {
    format!("/video/upload-subtitles/{}", video.site_path.normalized())
}

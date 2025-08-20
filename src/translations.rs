// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use crate::SitePath;

mod de;
mod en;

pub const DOCS_LINK: &str = "https://simonrepp.com/hyper8/docs";

#[derive(Clone, Debug)]
pub struct Translations {
    pub a_permalink_forms_part_of_text: &'static str,
    pub a_site_presenting_a_single_playlist_of_videos: &'static str,
    pub a_site_presenting_any_number_of_videos_and_playlists: &'static str,
    pub a_site_presenting_just_a_single_video: &'static str,
    pub about: &'static str,
    pub active: &'static str,
    pub adaptive: &'static str,
    pub add_link: &'static str,
    adding_file_xxx: &'static str,
    pub adjust_playback_speed_cursor_above_playback_speed_indicator: &'static str,
    pub advanced_settings: &'static str,
    pub already_deploying_can_only_deploy_again_when_current_deployment_is_finished: &'static str,
    pub arrow_left_right: &'static str,
    pub atom_description: &'static str,
    pub audio: &'static str,
    pub audio_files: &'static str,
    pub automatic_link_previews_and_content_embedding_on_platforms: &'static str,
    pub automatic_original_frame_rate_or_encoder_preference: &'static str,
    pub available_formats: &'static str,
    pub average_compression_and_speed_reasonable_for_temporarily_used_media: &'static str,
    pub back: &'static str,
    pub background_jobs: &'static str,
    pub banner_description: &'static str,
    pub banner_image: &'static str,
    pub beginning_deployment_over_secure_ftp_tls_ssl: &'static str,
    pub beginning_deployment_over_insecure_ftp_unencrypted: &'static str,
    pub best: &'static str,
    pub browse: &'static str,
    pub browser_support_unclear: &'static str,
    pub cancel: &'static str,
    pub clean_urls_label: &'static str,
    pub clear_search_or_close_browse_panel: &'static str,
    pub close: &'static str,
    pub codec: &'static str,
    pub collapse: &'static str,
    pub collection: &'static str,
    pub collection_manifest: &'static str,
    pub collection_settings: &'static str,
    pub configure: &'static str,
    pub configure_deployment: &'static str,
    pub content: &'static str,
    pub convert_to_collection: &'static str,
    pub convert_to_playlist: &'static str,
    pub convert_to_video: &'static str,
    pub cool: &'static str,
    pub copied: &'static str,
    pub copy: &'static str,
    pub copy_link: &'static str,
    pub copy_path: &'static str,
    pub copy_url: &'static str,
    could_not_compute_metadata_for_xxx: &'static str,
    pub create_playlist: &'static str,
    pub create_subcollection: &'static str,
    pub create_video: &'static str,
    creating_directory_xxx: &'static str,
    current_video_xxx_of_xxx_in_xxx: &'static str,
    pub dangerous_actions: &'static str,
    pub dark: &'static str,
    pub dark_color_scheme: &'static str,
    pub deep: &'static str,
    pub default: &'static str,
    pub default_sort_number_or_release_date_ascending: &'static str,
    pub delete: &'static str,
    pub delete_banner: &'static str,
    pub delete_collection: &'static str,
    pub delete_playlist: &'static str,
    pub delete_poster: &'static str,
    pub delete_video: &'static str,
    pub deleting_the_collection_text: &'static str,
    pub deleting_the_playlist_text: &'static str,
    pub deleting_the_video_text: &'static str,
    deleting_directory_xxx: &'static str,
    deleting_file_xxx: &'static str,
    pub deploy: &'static str,
    pub deploying: &'static str,
    pub deployment: &'static str,
    pub deployment_completed_successfully: &'static str,
    pub deployment_configuration_could_not_be_verified: &'static str,
    pub deployment_configuration_is_incomplete_can_not_deploy: &'static str,
    pub deployment_steps_successfully_determined_file_synchronization_will_now_commence: &'static str,
    pub ftp_config_password_label: &'static str,
    pub ftp_config_path_label: &'static str,
    pub ftp_config_port_label: &'static str,
    pub ftp_config_secure_label: &'static str,
    pub ftp_config_server_label: &'static str,
    pub ftp_config_user_label: &'static str,
    pub ftp_deployment: &'static str,
    pub description: &'static str,
    pub disabled: &'static str,
    pub documentation: &'static str,
    pub download: &'static str,
    pub download_permalink: &'static str,
    pub downloads: &'static str,
    pub embed: &'static str,
    pub embed_codes_permalink: &'static str,
    pub embedding: &'static str,
    pub empty: &'static str,
    pub empty_site: &'static str,
    pub enabled: &'static str,
    pub encode_audio: &'static str,
    pub encode_video: &'static str,
    pub encoding_approach_label: &'static str,
    pub enter_exit_fullscreen: &'static str,
    pub error: &'static str,
    pub expand: &'static str,
    pub external_link: &'static str,
    pub failed: &'static str,
    pub feed: &'static str,
    pub feeds: &'static str,
    pub filter: &'static str,
    pub focus_search_field: &'static str,
    pub frame_rate: &'static str,
    pub general: &'static str,
    pub generally_contains_codecs_supported_in_browsers: &'static str,
    pub generally_supported_in_browsers: &'static str,
    pub good_browser_support: &'static str,
    pub h264_supported_in_all_browsers_recommended: &'static str,
    pub hevc_h265_missing_browser_support_not_recommended: &'static str,
    pub hidden: &'static str,
    pub hidden_link_with_rel_me_attribute: &'static str,
    pub highest_compression_recommended_but_takes_a_long_time: &'static str,
    pub highest_speed_only_use_if_you_need_results_as_soon_as_possible: &'static str,
    pub hrs: &'static str,
    pub hyper_8_could_not_determine_whether_this_file_format_is_generally_compatible_with_browsers_or_not: &'static str,
    pub idle: &'static str,
    pub inherited: &'static str,
    pub initialize_site_as_a_collection: &'static str,
    pub initialize_site_as_a_playlist: &'static str,
    pub initialize_site_as_a_video: &'static str,
    pub invalid_site: &'static str,
    pub invalid_site_text: &'static str,
    pub invalid_url: &'static str,
    pub jump_back_forward_five_seconds: &'static str,
    pub jump_to_a_random_frame: &'static str,
    pub label: &'static str,
    pub label_can_not_be_empty: &'static str,
    pub label_optional: &'static str,
    pub language: &'static str,
    pub language_label: &'static str,
    pub length_diverges_between_different_formats: &'static str,
    pub light: &'static str,
    pub light_color_scheme: &'static str,
    pub link_previews_only: &'static str,
    pub link_previews_and_content_embeds: &'static str,
    pub link_type: &'static str,
    pub link_with_rel_me_attribute: &'static str,
    pub links: &'static str,
    pub local_path: &'static str,
    pub loud_volume: &'static str,
    pub markdown_inline_links_are_supported_standalone_urls_are_automatically_linked: &'static str,
    pub maximize: &'static str,
    pub media_rss_description: &'static str,
    pub medium_volume: &'static str,
    metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_currently_has_pending_errors: &'static str,
    metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_has_meanwhile_changed_path: &'static str,
    pub metadata_pending: &'static str,
    pub min: &'static str,
    pub minimize: &'static str,
    pub mousewheel: &'static str,
    pub move_into_new_collection: &'static str,
    pub move_into_new_playlist: &'static str,
    pub mp3_highest_compatibility: &'static str,
    pub mute_unmute_audio: &'static str,
    pub navigation: &'static str,
    pub neutral: &'static str,
    pub new_link: &'static str,
    pub new_playlist: &'static str,
    pub new_subcollection: &'static str,
    pub new_video: &'static str,
    pub no_audio: &'static str,
    pub no_language_info: &'static str,
    pub no_subtitles: &'static str,
    pub no_title: &'static str,
    pub no_video: &'static str,
    pub not_ok: &'static str,
    pub not_public: &'static str,
    pub not_saved: &'static str,
    pub not_supported_by_browsers: &'static str,
    pub nothing_found_for_xxx: &'static str,
    pub offline: &'static str,
    pub open_browse_panel: &'static str,
    pub opus_recommended_highest_efficiency: &'static str,
    pub order: &'static str,
    pub original: &'static str,
    pub pause: &'static str,
    pub pending_manifest_errors: &'static str,
    pub permalink: &'static str,
    pub permalink_already_used_by_another_collection_playlist_or_video: &'static str,
    pub pick_a_poster_from_the_video: &'static str,
    pub plain_rss: &'static str,
    pub plain_rss_description: &'static str,
    pub play: &'static str,
    pub play_pause_video: &'static str,
    pub playlist: &'static str,
    pub playlist_manifest: &'static str,
    pub playlist_settings: &'static str,
    pub podcast_rss_description: &'static str,
    pub poster_aspect_label: &'static str,
    pub poster_description: &'static str,
    pub poster_format: &'static str,
    pub poster_image: &'static str,
    pub preview: &'static str,
    pub preview_is_only_available_again_after_the_current_deployment_is_finished: &'static str,
    pub processing: &'static str,
    pub public_address_label: &'static str,
    pub public_address_must_be_configured_for_deployment_text: &'static str,
    pub public_address_not_configured_text: &'static str,
    pub quality: &'static str,
    pub really_delete: &'static str,
    pub really_deploy: &'static str,
    pub regular_link: &'static str,
    pub release_date: &'static str,
    pub release_date_ascending: &'static str,
    pub release_date_descending: &'static str,
    pub render_format: &'static str,
    pub replay: &'static str,
    pub report_a_bug: &'static str,
    pub resolution: &'static str,
    resolve_the_issue_on_disk_or_on_manifest_page: &'static str,
    pub resource_not_found: &'static str,
    pub rsync_deployment: &'static str,
    pub rsync_deployment_text: &'static str,
    pub search: &'static str,
    pub sec: &'static str,
    pub set_current_frame_as_poster: &'static str,
    pub shortcuts: &'static str,
    pub showing_xxx_results_for_xxx: &'static str,
    pub site_manifest: &'static str,
    pub site_order: &'static str,
    pub site_settings: &'static str,
    pub site_setup: &'static str,
    pub site_setup_change_note: &'static str,
    site_setup_text: &'static str,
    pub sort_number: &'static str,
    pub sort_number_ascending: &'static str,
    pub sort_number_descending: &'static str,
    pub sound_muted: &'static str,
    pub space: &'static str,
    pub ssh_config_path_label: &'static str,
    pub ssh_config_server_label: &'static str,
    pub ssh_config_user_label: &'static str,
    pub storage_usage: &'static str,
    pub subscribe: &'static str,
    pub subscribe_permalink: &'static str,
    pub subtitles: &'static str,
    pub synopsis: &'static str,
    test_file_successfully_uploaded_but_remote_verification_failed_message: &'static str,
    pub the_file_requested_for_deletion_was_not_found: &'static str,
    pub the_link_requested_for_deletion_was_not_found: &'static str,
    pub the_poster_to_be_updated_was_not_found: &'static str,
    the_resource_xxx_was_not_found_message: &'static str,
    the_subtitle_extension_of_xxx_is_not_supported_please_upload_only_srt_or_vtt_files: &'static str,
    pub the_subtitle_requested_for_deletion_was_not_found: &'static str,
    pub the_subtitles_to_be_updated_were_not_found: &'static str,
    the_test_file_contains_xxx_but_the_response_contained_xxx: &'static str,
    the_xxx_child_process_could_not_be_executed: &'static str,
    the_xxx_child_process_could_not_be_spawned: &'static str,
    the_xxx_child_process_terminated_unsuccessfully: &'static str,
    pub theme: &'static str,
    pub this_video_can_be_seamlessly_converted_to_a_video_in_a_collection: &'static str,
    pub this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist: &'static str,
    pub thumbnail_format: &'static str,
    pub title: &'static str,
    pub title_ascending: &'static str,
    pub title_descending: &'static str,
    pub title_or_permalink_needs_to_be_provided: &'static str,
    pub unlisted: &'static str,
    unsupported_file_extension_in_xxx: &'static str,
    pub update_collection: &'static str,
    pub update_description: &'static str,
    pub update_manifest: &'static str,
    pub update_permalink: &'static str,
    pub update_playlist: &'static str,
    pub update_site: &'static str,
    pub update_subtitles: &'static str,
    pub update_video: &'static str,
    updating_collection_from_editor_currently_not_allowed: &'static str,
    updating_file_xxx: &'static str,
    updating_playlist_from_editor_currently_not_allowed: &'static str,
    updating_site_from_editor_currently_not_allowed: &'static str,
    updating_video_from_editor_currently_not_allowed: &'static str,
    pub upload_banner: &'static str,
    pub upload_new_subtitles: &'static str,
    pub upload_poster: &'static str,
    pub upload_video_or_audio_file: &'static str,
    pub url: &'static str,
    pub version: &'static str,
    pub video: &'static str,
    pub video_audio_files: &'static str,
    pub video_files: &'static str,
    pub video_format: &'static str,
    pub video_manifest: &'static str,
    pub video_not_yet_available: &'static str,
    pub video_order: &'static str,
    pub video_player: &'static str,
    pub video_player_widget_for_xxx: &'static str,
    pub video_settings: &'static str,
    pub videos: &'static str,
    pub vp9_missing_browser_support_not_recommended: &'static str,
    pub warm: &'static str,
    pub warning: &'static str,
    pub website: &'static str,
    pub worst: &'static str,
    xxx_characters_maximum: &'static str,
    pub xxx_videos: &'static str,
    pub you_need_not_provide_both_a_title_and_permalink_but_one_of_the_two_must_be_provided: &'static str
}

impl Translations {
    pub fn adding_file_xxx(&self, path: &str) -> String {
        self.adding_file_xxx.replace("{path}", path)
    }

    pub fn could_not_compute_metadata_for_xxx(&self, path: &Path) -> String {
        self.could_not_compute_metadata_for_xxx.replace("{path}", &path.to_string_lossy())
    }

    pub fn creating_directory_xxx(&self, path: &str) -> String {
        self.creating_directory_xxx.replace("{path}", path)
    }

    pub fn current_video_xxx_of_xxx_in_xxx(
        &self,
        current: usize,
        parent: &str,
        total: usize
    ) -> String {
        self.current_video_xxx_of_xxx_in_xxx
            .replace("{current}", &current.to_string())
            .replace("{parent}", parent)
            .replace("{total}", &total.to_string())
    }

    pub fn deleting_directory_xxx(&self, path: &str) -> String {
        self.deleting_directory_xxx.replace("{path}", path)
    }

    pub fn deleting_file_xxx(&self, path: &str) -> String {
        self.deleting_file_xxx.replace("{path}", path)
    }

    pub fn invalid_site_text(&self) -> String {
        self.invalid_site_text.replace("{docs_link}", DOCS_LINK)
    }

    pub fn metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_currently_has_pending_errors(
        &self,
        site_path: &SitePath
    ) -> String {
        self.metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_currently_has_pending_errors
            .replace("{site_path}", site_path.normalized())
    }

    pub fn metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_has_meanwhile_changed_path(
        &self,
        site_path: &SitePath
    ) -> String {
        self.metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_has_meanwhile_changed_path
            .replace("{site_path}", site_path.normalized())
    }

    pub fn public_address_must_be_configured_for_deployment_text(
        &self,
        site_settings_url: &str
    ) -> String {
        self.public_address_must_be_configured_for_deployment_text
            .replace("{site_settings_url}", site_settings_url)
    }

    pub fn resolve_the_issue_on_disk_or_on_manifest_page(
        &self,
        manifest_url: &str
    ) -> String {
        self.resolve_the_issue_on_disk_or_on_manifest_page
            .replace("{manifest_url}", manifest_url)
    }

    pub fn site_setup_text(&self) -> String {
        self.site_setup_text.replace("{docs_link}", DOCS_LINK)
    }

    pub fn test_file_successfully_uploaded_but_remote_verification_failed_message(
        &self,
        error: &str,
        public_address: &str,
        remote_path: &str
    ) -> String {
        self.test_file_successfully_uploaded_but_remote_verification_failed_message
            .replace("{error}", error)
            .replace("{public_address}", public_address)
            .replace("{remote_path}", remote_path)
    }

    pub fn the_resource_xxx_was_not_found_message(&self, path: &str) -> String {
        self.the_resource_xxx_was_not_found_message.replace("{path}", path)
    }

    pub fn the_subtitle_extension_of_xxx_is_not_supported_please_upload_only_srt_or_vtt_files(
        &self,
        filename: &str
    ) -> String {
        self.the_subtitle_extension_of_xxx_is_not_supported_please_upload_only_srt_or_vtt_files.replace("{filename}", filename)
    }

    pub fn the_test_file_contains_xxx_but_the_response_contained_xxx(
        &self,
        expected: &str,
        encountered: &str
    ) -> String {
        self.the_test_file_contains_xxx_but_the_response_contained_xxx
            .replace("{encountered}", encountered)
            .replace("{expected}", expected)
    }

    pub fn the_xxx_child_process_could_not_be_executed(&self, command: &str) -> String {
        self.the_xxx_child_process_could_not_be_executed.replace("{command}", command)
    }

    pub fn the_xxx_child_process_could_not_be_spawned(&self, command: &str) -> String {
        self.the_xxx_child_process_could_not_be_spawned.replace("{command}", command)
    }

    pub fn the_xxx_child_process_terminated_unsuccessfully(&self, command: &str) -> String {
        self.the_xxx_child_process_terminated_unsuccessfully.replace("{command}", command)
    }

    pub fn unsupported_file_extension_in_xxx(&self, file_name: &str) -> String {
        self.unsupported_file_extension_in_xxx.replace("{file_name}", file_name)
    }

    pub fn updating_collection_from_editor_currently_not_allowed(
        &self,
        collection_title: &str,
        manifest_path: &str
    ) -> String {
        self.updating_collection_from_editor_currently_not_allowed
            .replace("{collection_title}", collection_title)
            .replace("{manifest_path}", manifest_path)
    }

    pub fn updating_file_xxx(&self, path: &str) -> String {
        self.updating_file_xxx.replace("{path}", path)
    }

    pub fn updating_playlist_from_editor_currently_not_allowed(
        &self,
        playlist_title: &str,
        manifest_path: &str
    ) -> String {
        self.updating_playlist_from_editor_currently_not_allowed
            .replace("{playlist_title}", playlist_title)
            .replace("{manifest_path}", manifest_path)
    }

    pub fn updating_site_from_editor_currently_not_allowed(
        &self,
        manifest_path: &str
    ) -> String {
        self.updating_site_from_editor_currently_not_allowed
            .replace("{manifest_path}", manifest_path)
    }

    pub fn updating_video_from_editor_currently_not_allowed(
        &self,
        video_title: &str,
        manifest_path: &str
    ) -> String {
        self.updating_video_from_editor_currently_not_allowed
            .replace("{video_title}", video_title)
            .replace("{manifest_path}", manifest_path)
    }

    pub fn video_player_widget_for_xxx(&self, title: &str) -> String {
        self.video_player_widget_for_xxx.replace("{title}", title)
    }

    pub fn xxx_characters_maximum(&self, maximum: usize) -> String {
        self.xxx_characters_maximum.replace("{maximum}", &maximum.to_string())
    }
}

#[test]
fn check_translations() {
    let locales = [
        Translations::DE,
        Translations::EN
    ];

    for translations in &locales {
        assert!(&translations.adding_file_xxx.contains("{path}"));
        assert!(&translations.creating_directory_xxx.contains("{path}"));
        assert!(&translations.could_not_compute_metadata_for_xxx.contains("{path}"));
        assert!(&translations.current_video_xxx_of_xxx_in_xxx.contains("{current}"));
        assert!(&translations.current_video_xxx_of_xxx_in_xxx.contains("{parent}"));
        assert!(&translations.current_video_xxx_of_xxx_in_xxx.contains("{total}"));
        assert!(&translations.deleting_file_xxx.contains("{path}"));
        assert!(&translations.deleting_directory_xxx.contains("{path}"));
        assert!(&translations.invalid_site_text.contains("{docs_link}"));
        assert!(&translations.metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_currently_has_pending_errors.contains("{site_path}"));
        assert!(&translations.metadata_for_video_xxx_could_not_be_assigned_in_the_site_tree_as_the_video_has_meanwhile_changed_path.contains("{site_path}"));
        assert!(&translations.public_address_must_be_configured_for_deployment_text.contains("{site_settings_url}"));
        assert!(&translations.resolve_the_issue_on_disk_or_on_manifest_page.contains("{manifest_url}"));
        assert!(&translations.site_setup_text.contains("{docs_link}"));
        assert!(&translations.test_file_successfully_uploaded_but_remote_verification_failed_message.contains("{error}"));
        assert!(&translations.test_file_successfully_uploaded_but_remote_verification_failed_message.contains("{public_address}"));
        assert!(&translations.test_file_successfully_uploaded_but_remote_verification_failed_message.contains("{remote_path}"));
        assert!(&translations.the_resource_xxx_was_not_found_message.contains("{path}"));
        assert!(&translations.the_subtitle_extension_of_xxx_is_not_supported_please_upload_only_srt_or_vtt_files.contains("{filename}"));
        assert!(&translations.the_test_file_contains_xxx_but_the_response_contained_xxx.contains("{encountered}"));
        assert!(&translations.the_test_file_contains_xxx_but_the_response_contained_xxx.contains("{expected}"));
        assert!(&translations.the_xxx_child_process_could_not_be_executed.contains("{command}"));
        assert!(&translations.the_xxx_child_process_could_not_be_spawned.contains("{command}"));
        assert!(&translations.the_xxx_child_process_terminated_unsuccessfully.contains("{command}"));
        assert!(&translations.unsupported_file_extension_in_xxx.contains("{file_name}"));
        assert!(&translations.updating_collection_from_editor_currently_not_allowed.contains("{collection_title}"));
        assert!(&translations.updating_collection_from_editor_currently_not_allowed.contains("{manifest_path}"));
        assert!(&translations.updating_file_xxx.contains("{path}"));
        assert!(&translations.updating_playlist_from_editor_currently_not_allowed.contains("{playlist_title}"));
        assert!(&translations.updating_playlist_from_editor_currently_not_allowed.contains("{manifest_path}"));
        assert!(&translations.updating_site_from_editor_currently_not_allowed.contains("{manifest_path}"));
        assert!(&translations.updating_video_from_editor_currently_not_allowed.contains("{video_title}"));
        assert!(&translations.updating_video_from_editor_currently_not_allowed.contains("{manifest_path}"));
        assert!(&translations.video_player_widget_for_xxx.contains("{title}"));
        assert!(&translations.xxx_characters_maximum.contains("{maximum}"));

        let disallowed_char = |c: char| !c.is_ascii_alphanumeric() && c != '-';

        assert!(!&translations.download_permalink.contains(disallowed_char));
        assert!(!&translations.embed_codes_permalink.contains(disallowed_char));
    }
}

// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use indoc::indoc;

use crate::{
    AspectRatio,
    Context,
    Collection,
    Degrees,
    Font,
    FtpConfig,
    GetAnyResult,
    HyperDir,
    InheritedOptions,
    Language,
    Percent,
    Playlist,
    SitePath,
    SiteUrl,
    SshConfig,
    SyncProtocol,
    Theme,
    ThemeMode,
    ThemePaint,
    Video
};
use crate::manifest::{attribute_error, element_error, not_supported_error};
use crate::util::supported_font_extension;

pub const DEFAULT_ROUNDNESS: u8 = 80;

const SITE_OPTIONS: &[&str] = &[
    "base_url",
    "disable_clean_urls",
    "font",
    "ftp_config",
    "language",
    "poster_aspect",
    "roundness",
    "ssh_config",
    "theme",
    "theme_mode",
    "theme.dark",
    "theme.light"
];

#[derive(Clone, Debug)]
pub struct Site {
    pub base_url: Option<SiteUrl>,
    pub clean_urls: bool,
    pub content: SiteContent,
    pub dark_theme: Option<ThemePaint>,
    pub font: Font,
    pub ftp_config: FtpConfig,
    pub language: Language,
    pub light_theme: Option<ThemePaint>,
    /// Syntactical and semantic errors inside the manifest for displaying to
    /// the user as well as to prevent programmatic updates to the site and
    /// its manifest while such errors are present.
    pub manifest_errors: Vec<String>,
    pub poster_aspect: AspectRatio,
    pub roundness: Option<Percent>,
    pub ssh_config: SshConfig,
    pub theme: Option<Theme>,
    pub theme_mode: Option<ThemeMode>
}

#[derive(Clone, Debug)]
pub enum SiteContent {
    Collection(Arc<Collection>),
    Empty,
    /// A site has invalid content when the basic specification of the data
    /// model is not adhered to, e.g. when the site directory contains both a
    /// collection.eno and a playlist.eno manifest. As such error conditions
    /// can only be manually created, they need to be manually resolved as
    /// well.
    Invalid { error: String },
    Playlist(Arc<Playlist>),
    Video(Arc<Video>)
}

impl Site {
    pub fn absolute_url_dir_unchecked(&self, dir: &str, site_path: &SitePath) -> String {
        self.base_url
            .as_ref()
            .unwrap()
            .join_dir(self, dir, site_path)
    }

    pub fn absolute_url_file_unchecked(&self, file: &str, site_path: &SitePath) -> String {
        self.base_url
            .as_ref()
            .unwrap()
            .join_file(file, site_path)
    }

    pub fn absolute_url_index(&self, site_path: &SitePath) -> Option<String> {
        self.base_url
            .as_ref()
            .map(|base_url| base_url.join_index(self, site_path))
    }

    pub fn absolute_url_index_unchecked(&self, site_path: &SitePath) -> String {
        self.base_url
            .as_ref()
            .map(|base_url| base_url.join_index(self, site_path))
            .unwrap()
    }

    pub fn absolute_url_prefix(&self, site_path: &SitePath) -> Option<String> {
        self.base_url
            .as_ref()
            .map(|base_url| base_url.join_prefix(site_path))
    }

    pub fn absolute_url_prefix_unchecked(&self, site_path: &SitePath) -> String {
        self.base_url
            .as_ref()
            .unwrap()
            .join_prefix(site_path)
    }

    pub fn absolute_url_unchecked(&self) -> &str {
        &self.base_url
            .as_ref()
            .unwrap()
            .normalized
    }

    pub fn all_metadata_computed(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.all_metadata_computed(),
            SiteContent::Playlist(playlist) => playlist.all_metadata_computed(),
            SiteContent::Video(video) => video.all_metadata_computed(),
            SiteContent::Empty |
            SiteContent::Invalid { .. } => true
        }
    }

    fn apply_manifest(&mut self, manifest_path: &Path) {
        let content = fs::read_to_string(manifest_path).unwrap();

        let document = match enolib::parse(&content) {
            Ok(document) => document,
            Err(err) => {
                self.manifest_errors.push(format!("Syntax error in {}:{} ({})", manifest_path.display(), err.line, err));
                return;
            }
        };

        for element in document.elements() {
            match element.key() {
                "base_url" => {
                    if let Some(value) = element.field_value() {
                        match SiteUrl::parse(value) {
                            Ok(url) => {
                                self.base_url = Some(url);
                            }
                            Err(err) => {
                                let message = format!("The supplied url seems to be malformed ({err})");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        }
                    } else {
                        let message = "base_url needs to be provided as a field with a value, e.g.: 'base_url: https://example.com'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "dark_theme" => {
                    if let Some(attributes) = element.attributes() {
                        let mut theme_paint = ThemePaint::default();

                        for attribute in attributes {
                            match attribute.key() {
                                "accent_brightening" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.accent_brightening = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_brightening value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_brightening needs to be provided as a field with a value, e.g.: 'accent_brightening = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "accent_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.accent_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_chroma needs to be provided as a field with a value, e.g.: 'accent_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "accent_hue" => {
                                    if let Some(value) = attribute.value() {
                                        match Degrees::try_from(value) {
                                            Ok(degrees) => {
                                                theme_paint.accent_hue = Some(degrees);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_hue value {value} was not recognized, only degree values from 0-360 degrees (with or without ° or deg as unit) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_hue needs to be provided as a field with a value, e.g.: 'accent_hue = 90deg'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "base_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.base_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The base_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "base_chroma needs to be provided as a field with a value, e.g.: 'base_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "base_hue" => {
                                    if let Some(value) = attribute.value() {
                                        match Degrees::try_from(value) {
                                            Ok(degrees) => {
                                                theme_paint.base_hue = Some(degrees);
                                            }
                                            Err(()) => {
                                                let message = format!("The base_hue value {value} was not recognized, only degree values from 0-360 degrees (with or without ° or deg as unit) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "base_hue needs to be provided as a field with a value, e.g.: 'base_hue = 90deg'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "contrast" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.contrast = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The contrast value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "contrast needs to be provided as a field with a value, e.g.: 'contrast = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "layered_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.layered_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The layered_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "layered_chroma needs to be provided as a field with a value, e.g.: 'layered_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                _ => {
                                    let recognized_keys = [
                                        "accent_brightening",
                                        "accent_chroma",
                                        "accent_hue",
                                        "base_chroma",
                                        "base_hue",
                                        "contrast",
                                        "layered_chroma"
                                    ];

                                    let recognized_keys = recognized_keys
                                        .iter()
                                        .map(|key| format!("'{key}'"))
                                        .collect::<Vec<_>>()
                                        .join(", ");

                                    let message = format!("The key/name of this attribute was not recognized, only {recognized_keys} are recognized inside a dark_theme field");
                                    let error = attribute_error(attribute, manifest_path, &message);
                                    self.manifest_errors.push(error);
                                }
                            }
                        }

                        self.dark_theme = Some(theme_paint);
                    } else {
                        let message = indoc!("
                            dark_theme needs to be provided as a field with attributes, e.g.:

                            dark_theme:
                            base_chroma = 20
                            base_hue = 90
                        ");

                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "disable_clean_urls" => {
                    if element.is_flag() {
                        self.clean_urls = false;
                    } else {
                        let message = "disable_clean_urls needs to be provided as a flag, that is, exactly as 'disable_clean_urls' (without colon and without value)";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "font" => {
                    if let Some(value) = element.field_value() {
                        let absolute_path = manifest_path.parent().unwrap().join(value);
                        if absolute_path.exists() {
                            if supported_font_extension(value) {
                                self.font = Font::Custom(value.to_string());
                            } else {
                                let message = "The provided font file extension is not supported, only woff and woff2 are";
                                let error = element_error(element, manifest_path, message);
                                self.manifest_errors.push(error);
                            }
                        } else {
                            let message = "The provided font file name is invalid - the file was not found";
                            let error = element_error(element, manifest_path, message);
                            self.manifest_errors.push(error);
                        }
                    } else {
                        let message = "font needs to be provided as a field with a value specifying the woff or woff2 font file name, e.g.: 'font: custom_sans.woff2'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "ftp_config" => {
                     if let Some(attributes) = element.attributes() {
                        for attribute in attributes {
                            match attribute.key() {
                                "mode" => {
                                    if let Some(value) = attribute.value() {
                                        match value {
                                            "insecure" => {
                                                self.ftp_config.secure = Some(false);
                                            }
                                            "secure" => {
                                                self.ftp_config.secure = Some(true);
                                            }
                                            other => {
                                                let message = format!("The mode option only supports the value 'insecure' or 'secure'. The given value '{other}' was not recognized.");
                                                let error = element_error(element, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                }
                                "password" => {
                                    if let Some(value) = attribute.value() {
                                        self.ftp_config.password = Some(value.to_string());
                                    }
                                }
                                "password_prompt" => {
                                    if let Some(value) = attribute.value() {
                                        match value {
                                            "disabled" => {
                                                self.ftp_config.password_prompt = Some(false);
                                            }
                                            "enabled" => {
                                                self.ftp_config.password_prompt = Some(true);
                                            }
                                            other => {
                                                let message = format!("The password_prompt option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                                let error = element_error(element, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                }
                                "path" => {
                                    if let Some(value) = attribute.value() {
                                        self.ftp_config.path = Some(value.to_string());
                                    }
                                }
                                "port" => {
                                    if let Some(value) = attribute.value() {
                                        match value.parse::<u16>() {
                                            Ok(number) => self.ftp_config.port = Some(number),
                                            Err(err) => {
                                                let message = format!("The port option only accepts positive integer numbers ({err})");
                                                let error = element_error(element, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                }
                                "server" => {
                                    if let Some(value) = attribute.value() {
                                        self.ftp_config.server = Some(value.to_string());
                                    }
                                }
                                "user" => {
                                    if let Some(value) = attribute.value() {
                                        self.ftp_config.user = Some(value.to_string());
                                    }
                                }
                                _ => {
                                    let message = "The key/name of this attribute was not recognized, only 'password', 'path', 'server' and 'user' are recognized inside an ftp_config field";
                                    let error = attribute_error(attribute, manifest_path, message);
                                    self.manifest_errors.push(error);
                                }
                            }
                        }
                    } else {
                        let message = indoc!("
                            ftp_config needs to be provided as a field with attributes, e.g.:

                            ftp_config:
                            password = SuchExquisitelyLongSecret
                            path = /data/web/user123/html
                            server = ftp-user123.example.com
                            user = user123
                        ");
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "language" => {
                    if let Some(value) = element.field_value() {
                        self.language = Language::from_code(value);
                    } else {
                        let message = "language needs to be provided as a field with a value, e.g.: 'language: de'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "light_theme" => {
                    if let Some(attributes) = element.attributes() {
                        let mut theme_paint = ThemePaint::default();

                        for attribute in attributes {
                            match attribute.key() {
                                "accent_brightening" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.accent_brightening = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_brightening value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_brightening needs to be provided as a field with a value, e.g.: 'accent_brightening = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "accent_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.accent_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_chroma needs to be provided as a field with a value, e.g.: 'accent_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "accent_hue" => {
                                    if let Some(value) = attribute.value() {
                                        match Degrees::try_from(value) {
                                            Ok(degrees) => {
                                                theme_paint.accent_hue = Some(degrees);
                                            }
                                            Err(()) => {
                                                let message = format!("The accent_hue value {value} was not recognized, only degree values from 0-360 degrees (with or without ° or deg as unit) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "accent_hue needs to be provided as a field with a value, e.g.: 'accent_hue = 90deg'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "base_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.base_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The base_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "base_chroma needs to be provided as a field with a value, e.g.: 'base_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "base_hue" => {
                                    if let Some(value) = attribute.value() {
                                        match Degrees::try_from(value) {
                                            Ok(degrees) => {
                                                theme_paint.base_hue = Some(degrees);
                                            }
                                            Err(()) => {
                                                let message = format!("The base_hue value {value} was not recognized, only degree values from 0-360 degrees (with or without ° or deg as unit) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "base_hue needs to be provided as a field with a value, e.g.: 'base_hue = 90deg'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "contrast" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.contrast = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The contrast value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "contrast needs to be provided as a field with a value, e.g.: 'contrast = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                "layered_chroma" => {
                                    if let Some(value) = attribute.value() {
                                        match Percent::try_from(value) {
                                            Ok(percent) => {
                                                theme_paint.layered_chroma = Some(percent);
                                            }
                                            Err(()) => {
                                                let message = format!("The layered_chroma value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                                let error = attribute_error(attribute, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        };
                                    } else {
                                        let message = "layered_chroma needs to be provided as a field with a value, e.g.: 'layered_chroma = 100%'";
                                        let error = attribute_error(attribute, manifest_path, message);
                                        self.manifest_errors.push(error);
                                    }
                                }
                                _ => {
                                    let recognized_keys = [
                                        "accent_brightening",
                                        "accent_chroma",
                                        "accent_hue",
                                        "base_chroma",
                                        "base_hue",
                                        "contrast",
                                        "layered_chroma"
                                    ];

                                    let recognized_keys = recognized_keys
                                        .iter()
                                        .map(|key| format!("'{key}'"))
                                        .collect::<Vec<_>>()
                                        .join(", ");

                                    let message = format!("The key/name of this attribute was not recognized, only {recognized_keys} are recognized inside a light_theme field");
                                    let error = attribute_error(attribute, manifest_path, &message);
                                    self.manifest_errors.push(error);
                                }
                            }
                        }

                        self.light_theme = Some(theme_paint);
                    } else {
                        let message = indoc!("
                            light_theme needs to be provided as a field with attributes, e.g.:

                            light_theme:
                            base_chroma = 20
                            base_hue = 90
                        ");

                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "poster_aspect" => {
                    if let Some(value) = element.field_value() {
                        match AspectRatio::from_representation(value) {
                            Ok(aspect_ratio) => {
                                self.poster_aspect = aspect_ratio;
                            }
                            Err(err) => {
                                let message = format!("The supplied aspect ratio was not recognized ({err})");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        }
                    } else {
                        let message = "poster_aspect needs to be provided as a field with a value, e.g.: 'poster_aspect: 16:9'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "roundness" => {
                    if let Some(value) = element.field_value() {
                        match Percent::try_from(value) {
                            Ok(percent) => {
                                self.roundness = Some(percent);
                            }
                            Err(()) => {
                                let message = format!("The roundness value {value} was not recognized, only integer percent values from 0-100% (with or without sign) are recognized");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        };
                    } else {
                        let message = "roundness needs to be provided as a field with a value, e.g.: 'roundness: 100%'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "deploy_config" |
                "ssh_config" => {
                    // TODO: Eventually implement a message about the
                    // deploy_config key being deprecated (or hard-deprecate
                    // it right away then), for now we rely on the option
                    // still silently working and being automatically
                    // upgraded through users saving the site settings in the
                    // editor.

                    if let Some(attributes) = element.attributes() {
                        for attribute in attributes {
                            match attribute.key() {
                                "password" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.password = Some(value.to_string());
                                    }
                                }
                                "password_prompt" => {
                                    if let Some(value) = attribute.value() {
                                        match value {
                                            "disabled" => {
                                                self.ssh_config.password_prompt = Some(false);
                                            }
                                            "enabled" => {
                                                self.ssh_config.password_prompt = Some(true);
                                            }
                                            other => {
                                                let message = format!("The password_prompt option only supports the value 'enabled' or 'disabled'. The given value '{other}' was not recognized.");
                                                let error = element_error(element, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                }
                                "path" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.path = Some(value.to_string());
                                    }
                                }
                                "port" => {
                                    if let Some(value) = attribute.value() {
                                        match value.parse::<u16>() {
                                            Ok(number) => self.ssh_config.port = Some(number),
                                            Err(err) => {
                                                let message = format!("The port option only accepts positive integer numbers ({err})");
                                                let error = element_error(element, manifest_path, &message);
                                                self.manifest_errors.push(error);
                                            }
                                        }
                                    }
                                }
                                "private_key" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.private_key = Some(PathBuf::from(value));
                                    }
                                }
                                "public_key" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.public_key = Some(PathBuf::from(value));
                                    }
                                }
                                "server" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.server = Some(value.to_string());
                                    }
                                }
                                "user" => {
                                    if let Some(value) = attribute.value() {
                                        self.ssh_config.user = Some(value.to_string());
                                    }
                                }
                                _ => {
                                    let message = "The key/name of this attribute was not recognized, only 'password', 'path', 'port', 'server' and 'user' are recognized inside an ssh_config field";
                                    let error = attribute_error(attribute, manifest_path, message);
                                    self.manifest_errors.push(error);
                                }
                            }
                        }
                    } else {
                        let message = indoc!("
                            ssh_config needs to be provided as a field with attributes, e.g.:

                            ssh_config:
                            path = /data/web/user123/html
                            server = ssh-user123.example.com
                            user = user123
                        ");
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "theme" => {
                    if let Some(value) = element.field_value() {
                        match Theme::from_key(value) {
                            Ok(theme) => {
                                self.theme = Some(theme);
                            }
                            Err(err) => {
                                let message = format!("The supplied theme key was no recognized ({err})");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        }
                    } else {
                        let message = "theme needs to be provided as a field with a value, e.g.: 'theme: light_warm'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                "theme_mode" => {
                    if let Some(value) = element.field_value() {
                        match ThemeMode::try_from(value) {
                            Ok(theme_mode) => {
                                self.theme_mode = Some(theme_mode);
                            }
                            Err(()) => {
                                let message = format!("The supplied theme mode was not recognized ('adaptive', 'dark' and 'light' are supported)");
                                let error = element_error(element, manifest_path, &message);
                                self.manifest_errors.push(error);
                            }
                        }
                    } else {
                        let message = "theme_mode needs to be provided as a field with a value, e.g.: 'theme_mode: dark'";
                        let error = element_error(element, manifest_path, message);
                        self.manifest_errors.push(error);
                    }
                }
                other => {
                    let message = not_supported_error("site.eno", other, &[SITE_OPTIONS]);
                    let error = element_error(element, manifest_path, &message);
                    self.manifest_errors.push(error);
                }
            }
        }
    }

    /// Returns storage used in bytes for the built site
    pub fn build_storage_usage(&self) -> u64 {
        match &self.content {
            SiteContent::Collection(collection) => collection.build_storage_usage(),
            SiteContent::Playlist(playlist) => playlist.build_storage_usage(),
            SiteContent::Video(video) => video.build_storage_usage(),
            _ => 0
        }
    }

    /// Traverses the site tree to determine if any video on the site has
    /// copy link enabled. This information is consecutively used to
    /// determine whether we need to write clipboard css/js assets to the
    /// build.
    pub fn copy_link_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.copy_link_requested(),
            SiteContent::Playlist(playlist) => playlist.copy_link_requested(),
            SiteContent::Video(video) => video.copy_link(),
            SiteContent::Empty |
            SiteContent::Invalid { .. } => false
        }
    }

    pub fn complete_sync_config(&self, protocol: SyncProtocol) -> bool {
        match protocol {
            SyncProtocol::Ftp => self.ftp_config.complete(),
            SyncProtocol::Ssh => self.ssh_config.complete()
        }
    }

    /// Traverses the site tree to determine if any published video on the
    /// site needs to render embed codes. This information is used to check
    /// whether we need to write clipboard css/js assets to the build.
    pub fn embed_codes_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.embed_codes_requested(),
            SiteContent::Playlist(playlist) => playlist.embed_codes_requested(),
            SiteContent::Video(video) => video.embedding(),
            SiteContent::Empty |
            SiteContent::Invalid { .. } => false
        }
    }

    /// Traverses the site tree to determine if any video on the site needs to
    /// render embeds. This information is used to check whether we need to
    /// write embed css/js assets to the build.
    pub fn embeds_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.embeds_requested(),
            SiteContent::Playlist(playlist) => playlist.embeds_requested(),
            SiteContent::Video(video) => video.embeds_requested(),
            SiteContent::Empty |
            SiteContent::Invalid { .. } => false
        }
    }

    /// Traverses the site tree to determine if any collection or playlist on
    /// the site needs to render feeds. This information is used to check
    /// whether we need to warn about base_url not being set and feeds
    /// therefore not being available in spite of being activated.
    pub fn feeds_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.feeds_requested(),
            SiteContent::Playlist(playlist) => playlist.feeds(),
            SiteContent::Empty |
            SiteContent::Invalid { .. } |
            SiteContent::Video(_) => false
        }
    }

    /// Returns a collection, playlist or video at the given site path.
    pub fn get_any(&self, site_path: &SitePath) -> GetAnyResult {
        if site_path.is_root_path() {
            match &self.content {
                SiteContent::Collection(collection) => GetAnyResult::Collection(Arc::clone(collection)),
                SiteContent::Playlist(playlist) => GetAnyResult::Playlist(Arc::clone(playlist)),
                SiteContent::Video(video) => GetAnyResult::Video(Arc::clone(video)),
                _ => GetAnyResult::None
            }
        } else {
            match &self.content {
                SiteContent::Collection(collection) => collection.get_any(site_path),
                SiteContent::Playlist(playlist) => playlist.get_any(site_path),
                _ => GetAnyResult::None
            }
        }
    }

    /// Returns the collection at the given site path.
    pub fn get_collection(&self, site_path: &SitePath) -> Option<Arc<Collection>> {
        if let SiteContent::Collection(collection) = &self.content {
            if site_path.is_root_path() {
                return Some(Arc::clone(collection));
            } else {
                return collection.get_collection(site_path);
            }
        }

        None
    }

    /// Returns the playlist at the given site path.
    pub fn get_playlist(&self, site_path: &SitePath) -> Option<Arc<Playlist>> {
        if site_path.is_root_path() {
            if let SiteContent::Playlist(playlist) = &self.content {
                return Some(Arc::clone(playlist));
            }
        } else if let SiteContent::Collection(collection) = &self.content {
            return collection.get_playlist(site_path)
        }

        None
    }

    /// Returns the video at the given site path.
    pub fn get_video(&self, site_path: &SitePath) -> Option<Arc<Video>> {
        if site_path.is_root_path() {
            if let SiteContent::Video(video) = &self.content {
                return Some(Arc::clone(video));
            }
        } else if let SiteContent::Playlist(playlist) = &self.content {
            return playlist.get_video(site_path)
        } else if let SiteContent::Collection(collection) = &self.content {
            return collection.get_video(site_path)
        }

        None
    }

    /// When we construct site-internal linking urls, we always
    /// append an index suffix. For instance we might build this:
    /// root_prefix ("../") + permalink ("foo") + index_suffix ("/") = "../foo/"
    /// If clean_urls is disabled however, we always append an index_suffix "/index.html",
    /// so that above example would result in "../foo/index.html".
    pub fn index_suffix(&self) -> &str {
        match self.clean_urls {
            true => "/",
            false => "/index.html"
        }
    }

    pub fn index_suffix_file_only(&self) -> &str {
        match self.clean_urls {
            true => "",
            false => "index.html"
        }
    }

    pub fn new() -> Site {
        Site {
            base_url: None,
            content: SiteContent::Empty,
            clean_urls: true,
            dark_theme: None,
            ftp_config: FtpConfig::new(),
            font: Font::Default,
            language: Language::default(),
            light_theme: None,
            manifest_errors: Vec::new(),
            poster_aspect: AspectRatio::Default,
            roundness: None,
            ssh_config: SshConfig::new(),
            theme: None,
            theme_mode: None
        }
    }

    /// Traverses the site tree to determine if platform_integration is
    /// enabled anywhere. This information is consecutively used to output a
    /// warning if base_url is not set (which is required for
    /// platform integration metadata to be generated).
    pub fn platform_integration_requested(&self) -> bool {
        match &self.content {
            SiteContent::Collection(collection) => collection.platform_integration_requested(),
            SiteContent::Playlist(playlist) => playlist.platform_integration_requested(),
            SiteContent::Video(video) => video.platform_integration_local().is_some_and(|platform_integration| platform_integration.enabled()),
            SiteContent::Empty |
            SiteContent::Invalid { .. } => false
        }
    }

    pub fn prompt_sync_password(&self, protocol: SyncProtocol) -> bool {
        match protocol {
            SyncProtocol::Ftp => matches!(self.ftp_config.password_prompt, Some(true)),
            SyncProtocol::Ssh => matches!(self.ssh_config.password_prompt, Some(true))
        }
    }

    pub fn read_dir(context: &Context, dir: &Path) -> Site {
        let hyper_dir = HyperDir::read(dir);

        let mut site = Site::new();

        if hyper_dir.site_manifest.is_some() {
            site.apply_manifest(hyper_dir.site_manifest.as_deref().unwrap());
        }

        let default_options = InheritedOptions::default();

        if hyper_dir.collection_manifest.is_some() {
            if hyper_dir.playlist_manifest.is_some() || hyper_dir.video_manifest.is_some() {
                let error = String::from("Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.");
                site.content = SiteContent::Invalid { error };
            } else {
                let collection = Collection::read_dir(context, &hyper_dir, default_options, &site);
                site.content = SiteContent::Collection(collection);
            }
        } else if hyper_dir.playlist_manifest.is_some() {
            if hyper_dir.video_manifest.is_some() {
                let error = String::from("Only one of collection.eno, playlist.eno and video.eno may exist in the site directory, site is not read until this is manually resolved.");
                site.content = SiteContent::Invalid { error };
            } else {
                let playlist = Playlist::read_dir(context, &hyper_dir, default_options, &site);
                site.content = SiteContent::Playlist(playlist);
            }
        } else if hyper_dir.video_manifest.is_some() {
            let video = Video::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Video(video);
        } else if !hyper_dir.dirs.is_empty() {
            let collection = Collection::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Collection(collection);
        } else if !hyper_dir.video_files.is_empty() {
            let video = Video::read_dir(context, &hyper_dir, default_options, &site);
            site.content = SiteContent::Video(video);
        }

        site
    }

    pub fn roundness(&self) -> u8 {
        self.roundness
            .map(|roundness| roundness.value())
            .unwrap_or(DEFAULT_ROUNDNESS)
    }

    pub fn theme(&self) -> Theme {
        self.theme.unwrap_or(Theme::default())
    }

    pub fn theme_mode(&self) -> ThemeMode {
        match self.theme() {
            Theme::AdaptiveCool => ThemeMode::Adaptive,
            Theme::AdaptiveDeep => ThemeMode::Adaptive,
            Theme::AdaptiveNeutral => ThemeMode::Adaptive,
            Theme::AdaptiveWarm => ThemeMode::Adaptive,
            Theme::DarkCool => ThemeMode::Dark,
            Theme::DarkDeep => ThemeMode::Dark,
            Theme::DarkNeutral => ThemeMode::Dark,
            Theme::DarkWarm => ThemeMode::Dark,
            Theme::Experimental => self.theme_mode.unwrap_or(ThemeMode::default()),
            Theme::LightCool => ThemeMode::Light,
            Theme::LightDeep => ThemeMode::Light,
            Theme::LightNeutral => ThemeMode::Light,
            Theme::LightWarm => ThemeMode::Light
        }
    }

    pub fn title(&self) -> &str {
        match &self.content {
            SiteContent::Collection(collection) => collection.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Playlist(playlist) => playlist.title.as_deref().unwrap_or("Hyper 8"),
            SiteContent::Video(video) => video.title.as_deref().unwrap_or("Hyper 8"),
            _ => "Hyper 8"
        }
    }

    /// Writes `site.eno` in the site directory
    pub fn write_manifest(&self, context: &Context) -> Result<(), String> {
        if !self.manifest_errors.is_empty() {
            let error = String::from("Writing to the manifest rejected as there are pending manifest errors");
            return Err(error);
        }

        let mut eno = String::new();

        if let Some(base_url) = &self.base_url {
            eno.push_str(&format!("base_url: {}\n", base_url.representation));
        }

        if !self.clean_urls {
            eno.push_str("disable_clean_urls\n");
        }

        if let Font::Custom(file_name) = &self.font {
            eno.push_str(&format!("font: {}\n", file_name));
        }

        if !self.language.default {
            eno.push_str(&format!("language: {}\n", self.language.code));
        }

        if let AspectRatio::Custom { representation, .. } = &self.poster_aspect {
            eno.push_str(&format!("poster_aspect: {}\n", representation));
        }

        if let Some(roundness) = &self.roundness {
            let roundness: String = roundness.into();
            eno.push_str(&format!("roundness: {roundness}\n"));
        }

        if let Some(theme) = &self.theme {
            eno.push_str(&format!("theme: {}\n", theme.key()));
        }

        if let Some(theme_mode) = &self.theme_mode {
            eno.push_str(&format!("theme_mode: {}\n", theme_mode.key()));
        }

        if let Some(dark_paint) = &self.dark_theme {
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str(&format!("dark_theme:\n"));

            if let Some(accent_brightening) = &dark_paint.accent_brightening {
                let accent_brightening: String = accent_brightening.into();
                eno.push_str(&format!("accent_brightening = {accent_brightening}\n"));
            }

            if let Some(accent_chroma) = &dark_paint.accent_chroma {
                let accent_chroma: String = accent_chroma.into();
                eno.push_str(&format!("accent_chroma = {accent_chroma}\n"));
            }

            if let Some(accent_hue) = &dark_paint.accent_hue {
                let accent_hue: String = accent_hue.into();
                eno.push_str(&format!("accent_hue = {accent_hue}\n"));
            }

            if let Some(base_chroma) = &dark_paint.base_chroma {
                let base_chroma: String = base_chroma.into();
                eno.push_str(&format!("base_chroma = {base_chroma}\n"));
            }

            if let Some(base_hue) = &dark_paint.base_hue {
                let base_hue: String = base_hue.into();
                eno.push_str(&format!("base_hue = {base_hue}\n"));
            }

            if let Some(contrast) = &dark_paint.contrast {
                let contrast: String = contrast.into();
                eno.push_str(&format!("contrast = {contrast}\n"));
            }

            if let Some(layered_chroma) = &dark_paint.layered_chroma {
                let layered_chroma: String = layered_chroma.into();
                eno.push_str(&format!("layered_chroma = {layered_chroma}\n"));
            }
        }

        if !self.ftp_config.is_empty() {
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str("ftp_config:\n");

            if let Some(secure) = self.ftp_config.secure {
                let mode = if secure { "secure" } else { "insecure" };
                eno.push_str(&format!("mode = {mode}\n"));
            }

            if let Some(password) = &self.ftp_config.password {
                eno.push_str(&format!("password = {password}\n"));
            }

            if let Some(enabled) = self.ftp_config.password_prompt {
                let mode = if enabled { "enabled" } else { "disabled" };
                eno.push_str(&format!("password_prompt = {mode}\n"));
            }

            if let Some(path) = &self.ftp_config.path {
                eno.push_str(&format!("path = {path}\n"));
            }

            if let Some(port) = &self.ftp_config.port {
                eno.push_str(&format!("port = {port}\n"));
            }

            if let Some(server) = &self.ftp_config.server {
                eno.push_str(&format!("server = {server}\n"));
            }

            if let Some(user) = &self.ftp_config.user {
                eno.push_str(&format!("user = {user}\n"));
            }
        }

        if let Some(light_paint) = &self.light_theme {
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str(&format!("light_theme:\n"));

            if let Some(accent_brightening) = &light_paint.accent_brightening {
                let accent_brightening: String = accent_brightening.into();
                eno.push_str(&format!("accent_brightening = {accent_brightening}\n"));
            }

            if let Some(accent_chroma) = &light_paint.accent_chroma {
                let accent_chroma: String = accent_chroma.into();
                eno.push_str(&format!("accent_chroma = {accent_chroma}\n"));
            }

            if let Some(accent_hue) = &light_paint.accent_hue {
                let accent_hue: String = accent_hue.into();
                eno.push_str(&format!("accent_hue = {accent_hue}\n"));
            }

            if let Some(base_chroma) = &light_paint.base_chroma {
                let base_chroma: String = base_chroma.into();
                eno.push_str(&format!("base_chroma = {base_chroma}\n"));
            }

            if let Some(base_hue) = &light_paint.base_hue {
                let base_hue: String = base_hue.into();
                eno.push_str(&format!("base_hue = {base_hue}\n"));
            }

            if let Some(contrast) = &light_paint.contrast {
                let contrast: String = contrast.into();
                eno.push_str(&format!("contrast = {contrast}\n"));
            }

            if let Some(layered_chroma) = &light_paint.layered_chroma {
                let layered_chroma: String = layered_chroma.into();
                eno.push_str(&format!("layered_chroma = {layered_chroma}\n"));
            }
        }

        if !self.ssh_config.is_empty() {
            if !eno.is_empty() { eno.push('\n'); }

            eno.push_str("ssh_config:\n");

            if let Some(password) = &self.ssh_config.password {
                eno.push_str(&format!("password = {password}\n"));
            }

            if let Some(enabled) = self.ssh_config.password_prompt {
                let mode = if enabled { "enabled" } else { "disabled" };
                eno.push_str(&format!("password_prompt = {mode}\n"));
            }

            if let Some(path) = &self.ssh_config.path {
                eno.push_str(&format!("path = {path}\n"));
            }

            if let Some(port) = &self.ssh_config.port {
                eno.push_str(&format!("port = {port}\n"));
            }

            if let Some(private_key) = &self.ssh_config.private_key {
                let private_key_display = private_key.display();
                eno.push_str(&format!("private_key = {private_key_display}\n"));
            }

            if let Some(public_key) = &self.ssh_config.public_key {
                let public_key_display = public_key.display();
                eno.push_str(&format!("public_key = {public_key_display}\n"));
            }

            if let Some(server) = &self.ssh_config.server {
                eno.push_str(&format!("server = {server}\n"));
            }

            if let Some(user) = &self.ssh_config.user {
                eno.push_str(&format!("user = {user}\n"));
            }
        }

        let manifest_path = context.site_dir.join("site.eno");

        fs::write(manifest_path, eno).map_err(|err| err.to_string())
    }
}

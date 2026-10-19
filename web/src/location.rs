// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_core::SitePath;

/// Represents the page we are currently viewing, i.e. some element in the
/// site tree or an auxiliary page like the jobs overview.
pub enum Location<'a> {
    AuxiliaryPage,
    SiteTree {
        site_path: &'a SitePath,
        subpage: Subpage
    }
}

#[derive(PartialEq)]
pub enum Subpage {
    Chapters,
    DangerousActions,
    Edit,
    Links,
    Manifest,
    Media,
    Poster,
    SiteManifest,
    SiteSettings,
    Subtitles
}

impl<'a> Location<'a> {
    pub fn active_subpage(&self, site_path: &SitePath) -> Option<&Subpage> {
        match self {
            Location::AuxiliaryPage => None,
            Location::SiteTree { site_path: location_site_path, subpage } => {
                match site_path == *location_site_path {
                    true => Some(subpage),
                    false => None
                }
            }
        }
    }

    pub fn chapters(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Chapters
        }
    }

    pub fn dangerous_actions(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::DangerousActions
        }
    }

    pub fn edit(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Edit
        }
    }

    pub fn intersects(&self, site_path: &SitePath) -> bool {
        match self {
            Location::AuxiliaryPage => false,
            Location::SiteTree { site_path: location_site_path, .. } => {
                location_site_path.starts_with(site_path)
            }
        }
    }

    pub fn links(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Links
        }
    }

    pub fn manifest(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Manifest
        }
    }

    pub fn media(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Media
        }
    }

    pub fn poster(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Poster
        }
    }

    pub fn site(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::SiteSettings
        }
    }

    pub fn site_manifest(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::SiteManifest
        }
    }

    pub fn subtitles(site_path: &SitePath) -> Location<'_> {
        Location::SiteTree {
            site_path,
            subpage: Subpage::Subtitles
        }
    }
}

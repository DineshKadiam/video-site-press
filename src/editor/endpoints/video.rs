// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

mod add_link;
mod dangerous_actions;
mod delete;
mod delete_link;
mod delete_poster;
mod delete_version;
mod downsize;
mod edit;
mod edit_manifest;
mod media;
mod r#move;
mod poster;
mod random_poster;
mod subtitles;
mod update;
mod update_manifest;
mod update_poster;
mod upload_poster;
mod upload_video;

pub use add_link::add_link;
pub use dangerous_actions::dangerous_actions;
pub use delete::delete;
pub use delete_link::delete_link;
pub use delete_poster::delete_poster;
pub use delete_version::delete_version;
pub use downsize::downsize;
pub use edit::edit;
pub use edit_manifest::edit_manifest;
pub use media::media;
pub use r#move::r#move;
pub use poster::poster;
pub use random_poster::random_poster;
pub use subtitles::subtitles;
pub use update::update;
pub use update_manifest::update_manifest;
pub use update_poster::update_poster;
pub use upload_poster::upload_poster;
pub use upload_video::upload_video;

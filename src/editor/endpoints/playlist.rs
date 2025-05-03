// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

mod add_link;
mod create_video;
mod dangerous_actions;
mod delete;
mod delete_banner;
mod delete_link;
mod edit;
mod edit_manifest;
mod r#move;
mod update;
mod update_banner;
mod update_manifest;
mod upload_banner;

pub use add_link::add_link;
pub use create_video::create_video;
pub use dangerous_actions::dangerous_actions;
pub use delete::delete;
pub use delete_banner::delete_banner;
pub use delete_link::delete_link;
pub use edit::edit;
pub use edit_manifest::edit_manifest;
pub use r#move::r#move;
pub use update::update;
pub use update_banner::update_banner;
pub use update_manifest::update_manifest;
pub use upload_banner::upload_banner;

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_session::Session;

use hyper8_core::{Language, Theme};

use crate::{ServerMode, TreeViewOrder};

/// An abstraction for editor settings like the current language, theme and
/// tree view order that conditionally obtains its data from one of two
/// sources ("backends"):
/// - With shared editor state there is a global ("singleton") session
///   state that is solely taken from our in-memory editor state (which is
///   initially read from disk and also persisted there whenever an editor
///   setting changes)
/// - With inherited editor state defaults, each client (device/browser) has its own session
///   that is tied to the client-server relationship (the server session and
///   cookies), in this case the in-memory editor state only provides defaults
///   initially or as long as a setting has not been specifically overridden
///   in the client-server session.
pub struct SessionState {
    /// Whether the splashscreen should automatically be displayed when the
    /// user hasn't seen it yet for the current version. (Background: When a
    /// single local user uses the editor through the desktop application,
    /// they already see the current version and related messaging around it
    /// in the launcher, hence the splashscreen should not again advertise
    /// these things)
    pub automatic_splash: bool,
    pub language: Language,
    pub theme: Theme,
    pub tree_view_order: TreeViewOrder
}

impl SessionState {
    pub fn all(
        server_mode: &ServerMode,
        session: &Session
    ) -> SessionState {
        match server_mode {
            ServerMode::MultipleRemoteUsers { inherited_editor_state } => {
                let language = match session.get::<String>("language_code") {
                    Ok(Some(code)) => Language::from_code(&code),
                    _ => inherited_editor_state.language.clone()
                };

                let theme = match session.get::<String>("theme_key") {
                    Ok(Some(key)) => Theme::from_key(&key).unwrap_or(Theme::default()),
                    _ => inherited_editor_state.theme.clone()
                };

                let tree_view_order = match session.get::<String>("tree_view_order_key") {
                    Ok(Some(key)) => TreeViewOrder::from_key(&key),
                    _ => inherited_editor_state.tree_view_order
                };

                SessionState {
                    automatic_splash: true,
                    language,
                    theme,
                    tree_view_order
                }
            }
            ServerMode::SingleLocalUser { synced_editor_state } => {
                let editor_state = synced_editor_state.lock().unwrap();
                let language = editor_state.language.clone();
                let theme = editor_state.theme.clone();
                let tree_view_order = editor_state.tree_view_order;

                SessionState {
                    automatic_splash: false,
                    language,
                    theme,
                    tree_view_order
                }
            }
        }
    }

    pub fn language(
        server_mode: &ServerMode,
        session: &Session
    ) -> Language {
        match server_mode {
            ServerMode::MultipleRemoteUsers { inherited_editor_state } => {
                match session.get::<String>("language_code") {
                    Ok(Some(code)) => Language::from_code(&code),
                    _ => inherited_editor_state.language.clone()
                }
            }
            ServerMode::SingleLocalUser { synced_editor_state } => {
                synced_editor_state.lock().unwrap().language.clone()
            }
        }
    }

    pub fn theme(
        server_mode: &ServerMode,
        session: &Session
    ) -> Theme {
        match server_mode {
            ServerMode::MultipleRemoteUsers { inherited_editor_state } => {
                match session.get::<String>("theme_key") {
                    Ok(Some(key)) => Theme::from_key(&key).unwrap_or(Theme::default()),
                    _ => inherited_editor_state.theme.clone()
                }
            }
            ServerMode::SingleLocalUser { synced_editor_state } => {
                synced_editor_state.lock().unwrap().theme.clone()
            }
        }
    }

    pub fn tree_view_order(
        server_mode: &ServerMode,
        session: &Session
    ) -> TreeViewOrder {
        match server_mode {
            ServerMode::MultipleRemoteUsers { inherited_editor_state } => {
                match session.get::<String>("tree_view_order_key") {
                    Ok(Some(key)) => TreeViewOrder::from_key(&key),
                    _ => inherited_editor_state.tree_view_order
                }
            }
            ServerMode::SingleLocalUser { synced_editor_state } => {
                synced_editor_state.lock().unwrap().tree_view_order
            }
        }
    }
}

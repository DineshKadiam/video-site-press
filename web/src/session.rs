// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use actix_session::Session as ActixSession;

use hyper8_core::{Language, Theme};

use crate::TreeViewOrder;

pub struct Session {
    pub language: Language,
    pub theme: Theme,
    pub tree_view_order: TreeViewOrder
}

impl Session {
    pub fn language(actix_session: &ActixSession) -> Language {
        match actix_session.get::<String>("language_code") {
            Ok(Some(code)) => Language::from_code(&code),
            _ => Language::default()
        }
    }

    pub fn new(actix_session: &ActixSession) -> Session {
        let language = Session::language(actix_session);
        let theme = Session::theme(actix_session);
        let tree_view_order = Session::tree_view_order(actix_session);

        Session {
            language,
            theme,
            tree_view_order
        }
    }

    pub fn theme(actix_session: &ActixSession) -> Theme {
        match actix_session.get::<String>("theme_key") {
            Ok(Some(key)) => {
                Theme::from_key(&key).unwrap_or(Theme::default())
            }
            _ => Theme::default()
        }
    }

    pub fn tree_view_order(actix_session: &ActixSession) -> TreeViewOrder {
        match actix_session.get::<String>("tree_view_order_key") {
            Ok(Some(key)) => {
                TreeViewOrder::from_key(&key).unwrap_or(TreeViewOrder::Default)
            }
            _ => TreeViewOrder::Default
        }
    }
}

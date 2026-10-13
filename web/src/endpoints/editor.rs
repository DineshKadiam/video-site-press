// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use serde_derive::Deserialize;

use hyper8_core::Language;

use crate::{
    Context,
    EditorTheme,
    ServerMode,
    SessionState
};

#[derive(Deserialize)]
pub struct LanguageForm {
    language_code: String,
    redirect: String
}

#[derive(Deserialize)]
pub struct ThemeForm {
    redirect: String,
    theme_key: String
}

#[derive(Deserialize)]
pub struct TreeViewOrderForm {
    redirect: String
}

pub async fn change_language(
    context: Data<Arc<Context>>,
    form: Form<LanguageForm>,
    session: Session
) -> Redirect {
    match &context.server_mode {
        ServerMode::MultipleRemoteUsers { .. } => {
            let _ = session.insert("language_code", &form.language_code);
        }
        ServerMode::SingleLocalUser { synced_editor_state } => {
            let language = Language::from_code(&form.language_code);
            synced_editor_state.lock().unwrap().set_language(language);
        }
    }

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn change_theme(
    context: Data<Arc<Context>>,
    form: Form<ThemeForm>,
    session: Session
) -> Redirect {
    match &context.server_mode {
        ServerMode::MultipleRemoteUsers { .. } => {
            let _ = session.insert("theme_key", &form.theme_key);
        }
        ServerMode::SingleLocalUser { synced_editor_state } => {
            let theme = EditorTheme::try_from(form.theme_key.as_str()).unwrap();
            synced_editor_state.lock().unwrap().set_theme(theme);
        }
    }

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn change_tree_view_order(
    context: Data<Arc<Context>>,
    form: Form<TreeViewOrderForm>,
    session: Session
) -> Redirect {
    match &context.server_mode {
        ServerMode::MultipleRemoteUsers { .. } => {
            let tree_view_order = SessionState::tree_view_order(&context.server_mode, &session);
            let _ = session.insert("tree_view_order_key", tree_view_order.cycle().key());
        }
        ServerMode::SingleLocalUser { synced_editor_state } => {
            synced_editor_state.lock().unwrap().cycle_tree_view_order();
        }
    }

    Redirect::to(form.redirect.clone()).see_other()
}

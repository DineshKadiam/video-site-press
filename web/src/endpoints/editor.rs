// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{Language, Theme};

use crate::{Context, EditorStateConnection, SessionState};

#[derive(Deserialize)]
pub struct LanguageAndThemeForm {
    language_code: String,
    redirect: String,
    theme_key: String
}

#[derive(Deserialize)]
pub struct TreeViewOrderForm {
    redirect: String
}

pub async fn change_language_and_theme(
    context: Data<Arc<Context>>,
    form: Form<LanguageAndThemeForm>,
    session: Session
) -> Redirect {
    match &context.editor_state {
        EditorStateConnection::InheritDefaults(_) => {
            let _ = session.insert("language_code", &form.language_code);
            let _ = session.insert("theme_key", &form.theme_key);
        }
        EditorStateConnection::Shared(editor_state) => {
            let mut editor_state_mut = editor_state.lock().unwrap();
            editor_state_mut.language = Language::from_code(&form.language_code);
            editor_state_mut.theme = Theme::from_key(&form.theme_key).unwrap();
            editor_state_mut.persist();
        }
    }

    Redirect::to(form.redirect.clone()).see_other()
}

pub async fn change_tree_view_order(
    context: Data<Arc<Context>>,
    form: Form<TreeViewOrderForm>,
    session: Session
) -> Redirect {
    match &context.editor_state {
        EditorStateConnection::InheritDefaults(_) => {
            let tree_view_order = SessionState::tree_view_order(&context.editor_state, &session);
            let _ = session.insert("tree_view_order_key", tree_view_order.cycle().key());
        }
        EditorStateConnection::Shared(editor_state) => {
            let mut editor_state_mut = editor_state.lock().unwrap();
            editor_state_mut.tree_view_order = editor_state_mut.tree_view_order.cycle();
            editor_state_mut.persist();
        }
    }

    Redirect::to(form.redirect.clone()).see_other()
}

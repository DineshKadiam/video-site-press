// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Deref;
use std::sync::Arc;

use hyper8_core::Context as CoreContext;

use crate::EditorStateConnection;

pub struct Context {
    pub core_context: Arc<CoreContext>,
    pub editor_state: EditorStateConnection
}

impl Context {
    pub fn new(
        core_context: Arc<CoreContext>,
        editor_state: EditorStateConnection
    ) -> Context {
        Context {
            core_context,
            editor_state
        }
    }
}

impl Deref for Context {
    type Target = CoreContext;

    fn deref(&self) -> &Self::Target {
        &self.core_context
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Deref;
use std::sync::Arc;

use hyper8_core::Context as CoreContext;

use crate::ServerMode;

pub struct Context {
    pub core_context: Arc<CoreContext>,
    pub server_mode: ServerMode
}

impl Context {
    pub fn new(
        core_context: Arc<CoreContext>,
        server_mode: ServerMode
    ) -> Context {
        Context {
            core_context,
            server_mode
        }
    }
}

impl Deref for Context {
    type Target = CoreContext;

    fn deref(&self) -> &Self::Target {
        &self.core_context
    }
}

// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Deref;
use std::sync::Arc;

use hyper8_core::Context as CoreContext;

// TODO: Keep around for a while because we might want to reintroduce proper
// editor-related state (e.g. in interplay with the desktop editor
// integration), but if we end up never introducing that, remove this abstraction
// and replace with simply just the core context.

pub struct Context {
    pub core_context: Arc<CoreContext>
}

impl Context {
    pub fn new(
        core_context: Arc<CoreContext>
    ) -> Context {
        Context {
            core_context
        }
    }
}

impl Deref for Context {
    type Target = CoreContext;

    fn deref(&self) -> &Self::Target {
        &self.core_context
    }
}

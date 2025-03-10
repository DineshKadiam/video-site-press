// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};

use crate::{Context, SitePath};
use crate::editor::widgets::not_found;

use super::edit_page;
use super::CollectionFormFeedback;

pub async fn edit(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let language = context.get_editor_language();

    match context.get_collection(&site_path) {
        Some(collection) => edit_page(&collection, &context, CollectionFormFeedback::None, &language),
        None => not_found(&context, &language, &site_path)
    }
}

// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};

use crate::{Context, ContextPath};
use crate::editor::widgets::not_found;

use super::edit_page;

pub async fn edit(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> HttpResponse {
    let language = context.get_editor_language();

    let video = match context.get_video(&path.path) {
        Some(video) => video,
        None => return not_found(&context, &language, &path.path)
    };

    edit_page(&context, None, &language, &video)
}

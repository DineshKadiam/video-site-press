// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, ContextPath};
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_playlist(&path.path) {
        let language = context.get_editor_language();
        return Either::Right(not_found(&context, &language, &path.path))
    };

    let redirect_url = if path.path.is_empty() {
        "/".to_string()
    } else {
        // We join("") to append a trailing slash (which parent() removes beforehand)
        Path::new("/collection/").join(&path.path).parent().unwrap().join("").to_str().unwrap().to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}

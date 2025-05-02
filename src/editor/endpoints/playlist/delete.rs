// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath};
use crate::editor::routes;
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.delete_playlist(&site_path) {
        Ok(Some(parent_collection)) => {
            let redirect_url = routes::collection_edit(&parent_collection);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Ok(None) => Either::Left(Redirect::to(routes::DASHBOARD).see_other()),
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
    }
}

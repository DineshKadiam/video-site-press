// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath};
use crate::context::VideoParent;
use crate::editor::routes;
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    match context.delete_video(&site_path) {
        Ok(Some(parent)) => {
            let redirect_url = match parent {
                VideoParent::Collection(collection) => routes::collection_edit(&collection),
                VideoParent::Playlist(playlist) => routes::playlist_edit(&playlist)
            };
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Ok(None) => Either::Left(Redirect::to(routes::DASHBOARD).see_other()),
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
    }
}
